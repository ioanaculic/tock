//! Driver for the MEMS L3GD20 motion sensor, 3 axys digital output gyroscope.
//!
//! <https://www.pololu.com/file/0J563/L3GD20.pdf>
//!
//! Usage
//! -----
//!
//! ```rust
//! let si7021_i2c = static_init!(
//!     capsules::virtual_i2c::I2CDevice,
//!     capsules::virtual_i2c::I2CDevice::new(i2c_bus, 0x40));
//! let si7021_virtual_alarm = static_init!(
//!     VirtualMuxAlarm<'static, sam4l::ast::Ast>,
//!     VirtualMuxAlarm::new(mux_alarm));
//! let si7021 = static_init!(
//!     capsules::si7021::SI7021<'static, VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
//!     capsules::si7021::SI7021::new(si7021_i2c,
//!         si7021_virtual_alarm,
//!         &mut capsules::si7021::BUFFER));
//! si7021_i2c.set_client(si7021);
//! si7021_virtual_alarm.set_client(si7021);
//! ```

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::debug;
use kernel::hil::gpio;
use kernel::hil::spi;
use kernel::hil::time;
use kernel::hil::time::Frequency;
use kernel::Driver;
use kernel::ReturnCode;

use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::L3gd20 as usize;

/* Identification number */
const L3GD20_WHO_AM_I: u8 = 0xD4;

/* Registers addresses */
const L3GD20_REG_WHO_AM_I: u8 = 0x0F;
const L3GD20_REG_CTRL_REG1: u8 = 0x20;
const L3GD20_REG_CTRL_REG2: u8 = 0x21;
const L3GD20_REG_CTRL_REG3: u8 = 0x22;
const L3GD20_REG_CTRL_REG4: u8 = 0x23;
const L3GD20_REG_CTRL_REG5: u8 = 0x24;
const L3GD20_REG_REFERENCE: u8 = 0x25;
const L3GD20_REG_OUT_TEMP: u8 = 0x26;
const L3GD20_REG_STATUS_REG: u8 = 0x27;
const L3GD20_REG_OUT_X_L: u8 = 0x28;
const L3GD20_REG_OUT_X_H: u8 = 0x29;
const L3GD20_REG_OUT_Y_L: u8 = 0x2A;
const L3GD20_REG_OUT_Y_H: u8 = 0x2B;
const L3GD20_REG_OUT_Z_L: u8 = 0x2C;
const L3GD20_REG_OUT_Z_H: u8 = 0x2D;
const L3GD20_REG_FIFO_CTRL_REG: u8 = 0x2E;
const L3GD20_REG_FIFO_SRC_REG: u8 = 0x2F;
const L3GD20_REG_INT1_CFG: u8 = 0x30;
const L3GD20_REG_INT1_SRC: u8 = 0x31;
const L3GD20_REG_INT1_TSH_XH: u8 = 0x32;
const L3GD20_REG_INT1_TSH_XL: u8 = 0x33;
const L3GD20_REG_INT1_TSH_YH: u8 = 0x34;
const L3GD20_REG_INT1_TSH_YL: u8 = 0x35;
const L3GD20_REG_INT1_TSH_ZH: u8 = 0x36;
const L3GD20_REG_INT1_TSH_ZL: u8 = 0x37;
const L3GD20_REG_INT1_DURATION: u8 = 0x38;

pub const L3GD20_TX_SIZE: usize = 100;
pub const L3GD20_RX_SIZE: usize = 100;

/* Sensitivity factors, datasheet pg. 9 */
const L3GD20_SENSITIVITY_250: f32 = 8.75; /* 8.75 mdps/digit */
const L3GD20_SENSITIVITY_500: f32 = 17.5; /* 17.5 mdps/digit */
const L3GD20_SENSITIVITY_2000: f32 = 70.0; /* 70 mdps/digit */

pub enum SenzitivityScale {
	S_250,
	S_500,
	S_2000,
}

// #[derive(Clone, Copy, PartialEq)]
// enum State {
//     Idle,
// }

pub struct L3GD20<'a, A: time::Alarm<'a>> {
	spi: &'a dyn spi::SpiMasterDevice,
	alarm: &'a A,
	txbuffer: TakeCell<'static, [u8]>,
	rxbuffer: TakeCell<'static, [u8]>,
}

impl<A: time::Alarm<'a>> L3GD20<'a, A> {
	pub fn new(
		spi: &'a dyn spi::SpiMasterDevice,
		alarm: &'a A,
		txbuffer: &'static mut [u8; L3GD20_TX_SIZE],
		rxbuffer: &'static mut [u8; L3GD20_RX_SIZE],
	) -> L3GD20<'a, A> {
		// setup and return struct
		L3GD20 {
			spi: spi,
			alarm: alarm,
			txbuffer: TakeCell::new(txbuffer),
			rxbuffer: TakeCell::new(rxbuffer),
		}
	}

	pub fn is_present(&self) -> bool {
		self.txbuffer.take().map (|buf| {
			buf[0] = L3GD20_REG_WHO_AM_I | 0x80;
			buf[1] = 0xFF;
			self.spi.read_write_bytes (buf, self.rxbuffer.take(), 2);
		});
		false
	}

	pub fn configure (&self)
	{
		self.spi.configure(
            spi::ClockPolarity::IdleHigh,
            spi::ClockPhase::SampleTrailing,
            1_000_000,
        );
	}
}

impl<A: time::Alarm<'a>> time::AlarmClient for L3GD20<'a, A> {
	fn fired(&self) {}
}

impl<A: time::Alarm<'a>> Driver for L3GD20<'a, A> {}

impl<A: time::Alarm<'a>> spi::SpiMasterClient for L3GD20<'a, A> {
	fn read_write_done(
		&self,
		write_buffer: &'static mut [u8],
		read_buffer: Option<&'static mut [u8]>,
		len: usize,
	) {
		debug!("read_buffer {:?}", read_buffer);
	}
}
