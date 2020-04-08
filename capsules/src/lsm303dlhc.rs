//! Driver for the LSM303DLHC 3D accelerometer and 3D magnetometer sensor.
//!
//! <https://www.st.com/en/mems-and-sensors/lsm303dlhc.html>
//!
//! Usage
//! -----
//!
//! ```rust
//! let lps25hb_i2c = static_init!(I2CDevice, I2CDevice::new(i2c_bus, 0x5C));
//! let lps25hb = static_init!(
//!     capsules::lps25hb::LPS25HB<'static>,
//!     capsules::lps25hb::LPS25HB::new(lps25hb_i2c,
//!         &sam4l::gpio::PA[10],
//!         &mut capsules::lps25hb::BUFFER));
//! lps25hb_i2c.set_client(lps25hb);
//! sam4l::gpio::PA[10].set_client(lps25hb);
//! ```

use core::cell::Cell;
use enum_primitive::cast::FromPrimitive;
use enum_primitive::cast::ToPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::debug;
use kernel::hil::gpio;
use kernel::hil::i2c::{self, Error};
use kernel::{AppId, Callback, Driver, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Lsm303dlch as usize;

// Buffer to use for I2C messages
pub static mut BUFFER: [u8; 5] = [0; 5];

/// Register values
const REGISTER_AUTO_INCREMENT: u8 = 0x80;

/// Registers
const CTRL_REG1: u8 = 0x20;
const CTRL_REG4: u8 = 0x23;
const OUT_X_L_A: u8 = 0x28;
const OUT_X_H_A: u8 = 0x29;
const OUT_Y_L_A: u8 = 0x2A;
const OUT_Y_H_A: u8 = 0x2B;
const OUT_Z_L_A: u8 = 0x2C;
const OUT_Z_H_A: u8 = 0x2D;

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum Lsm303dlhcDataRate {
        Off = 0,
        DataRate1Hz = 1,
        DataRate10Hz = 2,
        DataRate25Hz = 3,
        DataRate50Hz = 4,
        DataRate100Hz = 5,
        DataRate200Hz = 6,
        DataRate400Hz = 7,
        LowPower1620Hz = 8,
        Normal1344LowPower5376Hz = 9,
    }
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum Lsm303dlhcScale {
        Scale2G = 0,
        Scale4G = 1,
        Scale8G = 2,
        Scale16G = 3
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    IsPresent,
    SetPowerMode,
    SetScaleAndResolution,
    ReadAccelerationXYZ,
}

pub struct Lsm303dlhc<'a> {
    i2c_accelerometer: &'a dyn i2c::I2CDevice,
    i2c_magnetometer: &'a dyn i2c::I2CDevice,
    callback: OptionalCell<Callback>,
    state: Cell<State>,
    scale: Cell<Lsm303dlhcScale>,
    high_resolution: Cell<bool>,
    buffer: TakeCell<'static, [u8]>,
}

impl Lsm303dlhc<'a> {
    pub fn new(
        i2c_accelerometer: &'a dyn i2c::I2CDevice,
        i2c_magnetometer: &'a dyn i2c::I2CDevice,
        buffer: &'static mut [u8],
    ) -> Lsm303dlhc<'a> {
        // setup and return struct
        Lsm303dlhc {
            i2c_accelerometer: i2c_accelerometer,
            i2c_magnetometer: i2c_magnetometer,
            callback: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            scale: Cell::new(Lsm303dlhcScale::Scale2G),
            high_resolution: Cell::new(false),
            buffer: TakeCell::new(buffer),
        }
    }

    fn is_present(&self) {
        self.state.set(State::IsPresent);
        self.buffer.take().map(|buf| {
            // turn on i2c to send commands
            buf[0] = 0x0F;
            self.i2c_magnetometer.write_read(buf, 1, 1);
        });
    }

    pub fn set_power_mode(&self, data_rate: Lsm303dlhcDataRate, low_power: bool) {
        if self.state.get() == State::Idle {
            self.state.set(State::SetPowerMode);
            self.buffer.take().map(|buf| {
                buf[0] = CTRL_REG1;
                buf[1] = ((data_rate as u8) << 4) | if low_power { 1 << 3 } else { 0 } | 0x7;
                self.i2c_accelerometer.write(buf, 2);
            });
        }
    }

    pub fn set_scale_and_resolution(&self, scale: Lsm303dlhcScale, high_resolution: bool) {
        if self.state.get() == State::Idle {
            self.state.set(State::SetScaleAndResolution);
            // TODO move these in completed
            self.scale.set(scale);
            self.high_resolution.set(high_resolution);
            self.buffer.take().map(|buf| {
                buf[0] = CTRL_REG4;
                buf[1] = (scale as u8) << 4 | if high_resolution { 1 } else { 0 } << 3;
                self.i2c_accelerometer.write(buf, 2);
            });
        }
    }

    pub fn read_acceleration_xyz(&self) {
        if self.state.get() == State::Idle {
            self.state.set(State::ReadAccelerationXYZ);
            self.buffer.take().map(|buf| {
                buf[0] = OUT_X_L_A | REGISTER_AUTO_INCREMENT;
                self.i2c_accelerometer.write_read(buf, 1, 6);
            });
        }
    }
}

impl i2c::I2CClient for Lsm303dlhc<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], error: Error) {
        match self.state.get() {
            State::IsPresent => {
                let present = if error == Error::CommandComplete && buffer[0] == 60 {
                    true
                } else {
                    false
                };

                self.callback.map(|callback| {
                    callback.schedule(if present { 1 } else { 0 }, 0, 0);
                });
            }
            State::SetPowerMode => {
                let set_power = error == Error::CommandComplete;

                self.callback.map(|callback| {
                    callback.schedule(if set_power { 1 } else { 0 }, 0, 0);
                });
            }
            State::SetScaleAndResolution => {
                let set_scale_and_resolution = error == Error::CommandComplete;

                self.callback.map(|callback| {
                    callback.schedule(if set_scale_and_resolution { 1 } else { 0 }, 0, 0);
                });
            }
            State::ReadAccelerationXYZ => {
                let mut x: usize = 0;
                let mut y: usize = 0;
                let mut z: usize = 0;
                let values = if error == Error::CommandComplete {
                    // self.nine_dof_client.map(|client| {
                    //     // compute using only integers
                    //     let scale = match self.scale.get() {
                    //         0 => L3GD20_SCALE_250,
                    //         1 => L3GD20_SCALE_500,
                    //         _ => L3GD20_SCALE_2000,
                    //     };
                    //     let x: usize = ((buf[1] as i16 | ((buf[2] as i16) << 8)) as isize * scale
                    //         / 100000) as usize;
                    //     let y: usize = ((buf[3] as i16 | ((buf[4] as i16) << 8)) as isize * scale
                    //         / 100000) as usize;
                    //     let z: usize = ((buf[5] as i16 | ((buf[6] as i16) << 8)) as isize * scale
                    //         / 100000) as usize;
                    //     client.callback(x, y, z);
                    // });
                    // actiual computation is this one

                    x = (buffer[0] as i16 | ((buffer[1] as i16) << 8)) as usize;
                    y = (buffer[2] as i16 | ((buffer[3] as i16) << 8)) as usize;
                    z = (buffer[4] as i16 | ((buffer[5] as i16) << 8)) as usize;
                    true
                } else {
                    // self.nine_dof_client.map(|client| {
                    //     client.callback(0, 0, 0);
                    // });
                    false
                };
                if values {
                    self.callback.map(|callback| {
                        callback.schedule(x, y, z);
                    });
                } else {
                    self.callback.map(|callback| {
                        callback.schedule(0, 0, 0);
                    });
                }
            }
            _ => {
                debug!("buffer {:?} error {:?}", buffer, error);
            }
        }
        self.buffer.replace(buffer);
        self.state.set(State::Idle);
    }
}

impl Driver for Lsm303dlhc<'a> {
    fn command(&self, command_num: usize, data1: usize, data2: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            // Check is sensor is correctly connected
            1 => {
				if self.state.get () == State::Idle {
					self.is_present ();
					ReturnCode::SUCCESS
				}
				else
				{
					ReturnCode::EBUSY
				}

			}
			// Set Power Mode
            2 => {
				if self.state.get () == State::Idle {
                    if let Some (data_rate) = Lsm303dlhcDataRate::from_usize (data1) {
                        self.set_power_mode(data_rate, if data2 != 0 { true } else { false });
                        ReturnCode::SUCCESS
                    }
                    else
                    {
                        ReturnCode::EINVAL
                    }
				}
				else
				{
					ReturnCode::EBUSY
				}
			}
			// Set Scale And Resolution
            3 => {
				if self.state.get () == State::Idle {
					if let Some (scale) = Lsm303dlhcScale::from_usize(data1) {
                        self.set_scale_and_resolution(scale, if data2 != 0 { true } else { false });
                        ReturnCode::SUCCESS
                    }
                    else
                    {
                        ReturnCode::EINVAL
                    }
				}
				else
				{
					ReturnCode::EBUSY
				}
            }
			// // Enable High Pass Filter
            // 4 => {
			// 	if self.status.get () == L3gd20Status::Idle {
			// 		let mode = data1 as u8;
			// 		let divider = data2 as u8;
			// 		self.set_hpf_parameters (mode, divider);
			// 		ReturnCode::SUCCESS
			// 	}
			// 	else
			// 	{
			// 		ReturnCode::EBUSY
			// 	}
			// }
			// // Set High Pass Filter Mode and Divider
            // 5 => {
			// 	if self.status.get () == L3gd20Status::Idle {
			// 		let enabled = if data1 == 1 { true } else { false };
			// 		self.enable_hpf (enabled);
			// 		ReturnCode::SUCCESS
			// 	}
			// 	else
			// 	{
			// 		ReturnCode::EBUSY
			// 	}
            // }
			// Read Acceleration XYZ
            6 => {
				if self.state.get () == State::Idle {
					self.read_acceleration_xyz ();
					ReturnCode::SUCCESS
				}
				else
				{
					ReturnCode::EBUSY
				}
			}
			// // Read Temperature
            // 7 => {
			// 	if self.status.get () == L3gd20Status::Idle {
			// 		self.read_temperature ();
			// 		ReturnCode::SUCCESS
			// 	}
			// 	else
			// 	{
			// 		ReturnCode::EBUSY
			// 	}
            // }
            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        _app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 /* set the one shot callback */ => {
				self.callback.insert (callback);
				ReturnCode::SUCCESS
			},
            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
