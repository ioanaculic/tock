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
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::debug;
use kernel::hil::gpio;
use kernel::hil::i2c;
use kernel::{AppId, Callback, Driver, ReturnCode};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Lsm303dlch as usize;

// Buffer to use for I2C messages
pub static mut BUFFER: [u8; 5] = [0; 5];

/// Register values
const REGISTER_AUTO_INCREMENT: u8 = 0x80;

const CTRL_REG1_POWER_ON: u8 = 0x80;
const CTRL_REG1_BLOCK_DATA_ENABLE: u8 = 0x04;
const CTRL_REG2_ONE_SHOT: u8 = 0x01;
const CTRL_REG4_INTERRUPT1_DATAREADY: u8 = 0x01;

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
}

pub struct Lsm303dlhc<'a> {
    i2c: &'a dyn i2c::I2CDevice,
    callback: OptionalCell<Callback>,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
}

impl Lsm303dlhc<'a> {
    pub fn new(i2c: &'a dyn i2c::I2CDevice, buffer: &'static mut [u8]) -> Lsm303dlhc<'a> {
        // setup and return struct
        Lsm303dlhc {
            i2c: i2c,
            callback: OptionalCell::empty(),
            state: Cell::new(State::Idle),
            buffer: TakeCell::new(buffer),
        }
    }

    pub fn is_present(&self) {
        self.buffer.take().map(|buf| {
            // turn on i2c to send commands
            self.i2c.enable();

            buf[0] = 0x0F;
            self.i2c.write_read(buf, 1, 1);
        });
    }
}

impl i2c::I2CClient for Lsm303dlhc<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: i2c::Error) {
        debug!("buffer {:?} error {:?}", buffer, _error);
    }
}

impl Driver for Lsm303dlhc<'a> {}
