//! Components for I2C.
//!
//! This provides three components.
//!
//! 1. `I2CMuxComponent` provides a virtualization layer for a I2C bus.
//!
//! 2. `I2CSyscallComponent` provides a system call interface to I2C.
//!
//! 3. `I2CComponent` provides a virtualized client to the I2C bus.
//!
//! `SpiSyscallComponent` is used for processes, while `I2CComponent` is used
//! for kernel capsules that need access to the SPI bus.
//!
//! Usage
//! -----
//! ```rust
//! let mux_i2c = components::i2c::I2CMuxComponent::new().finalize(
//!     components::i2c_mux_component_helper!(stm32f3xx::i2c::I2C1));
//! let i2c_syscalls = I2CyscallComponent::new(mux_i2c, address).finalize(
//!     components::i2c_syscalls_component_helper!(stm32f3xx::i2c::I2C1));
//! let client_i2c = I2CComponent::new(mux_i2c, address).finalize(
//!     components::spi_component_helper!(stm32f3xx::i2c::I2C1));
//! ```

// Author: Alexandru Radovici <msg4alex@gmail.com>

use core::mem::MaybeUninit;

use capsules::i2c_master::I2CMasterDriver;
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use kernel::component::Component;
use kernel::hil::i2c;
use kernel::static_init_half;

// Setup static space for the objects.
// #[macro_export]
// macro_rules! i2c_mux_component_helper {
// 	($S:ty) => {{
// 		use core::mem::MaybeUninit;
// 		static mut BUF: MaybeUninit<MuxI2C<'static>> = MaybeUninit::uninit();
// 		&mut BUF
// 		};};
// }

#[macro_export]
macro_rules! i2c_syscall_component_helper {
	($S:ty) => {{
		use capsules::i2c::I2CMasterDriver;
		use core::mem::MaybeUninit;
		static mut BUF1: MaybeUninit<I2CDevice<'static>> = MaybeUninit::uninit();
		static mut BUF2: MaybeUninit<I2CMasterDriver<'static, I2CDevice<'static>>> =
			MaybeUninit::uninit();
		(&mut BUF1, &mut BUF2)
		};};
}

#[macro_export]
macro_rules! i2c_component_helper {
	($S:ty) => {{
		use core::mem::MaybeUninit;
		static mut BUF: MaybeUninit<I2CDevice<'static>> = MaybeUninit::uninit();
		&mut BUF
		};};
}

pub struct I2CMuxComponent {
	i2c: &'static dyn i2c::I2CMaster,
}

pub struct I2CSyscallComponent {
	i2c_mux: &'static MuxI2C<'static>,
	address: u8,
}

// pub struct I2CComponent<S: 'static + i2c::I2CMaster> {
// 	i2c_mux: &'static MuxI2C<'static>,
// 	address: u8,
// }

impl I2CMuxComponent {
	pub fn new(i2c: &'static dyn i2c::I2CMaster) -> Self {
		I2CMuxComponent { i2c: i2c }
	}
}

impl Component for I2CMuxComponent {
	type StaticInput = &'static dyn i2c::I2CMaster>;
	type Output = &'static MuxI2C<'static>;

	unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
		let mux_i2c = static_init!(MuxI2C<'static>, MuxI2C::new(self.i2c));

		self.i2c.set_client(mux_i2c);
		self.i2c.init();

		mux_i2c
	}
}

// impl<S: 'static + i2c::I2CMaster> I2CSyscallComponent<S> {
// 	pub fn new(mux: &'static MuxI2C<'static>, address: u8) -> Self {
// 		I2CSyscallComponent {
// 			i2c_mux: mux,
// 			address: address,
// 		}
// 	}
// }

// impl<S: 'static + i2c::I2CMaster> Component for I2CSyscallComponent<S> {
// 	type StaticInput = (
// 		&'static mut MaybeUninit<I2CDevice<'static>>,
// 		&'static mut MaybeUninit<I2CMasterDriver<'static, I2CDevice<'static>>>,
// 	);
// 	type Output = &'static I2CMasterDriver<'static, I2CDevice<'static>>;

// 	unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
// 		let syscall_i2c_device = static_init_half!(
// 			static_buffer.0,
// 			I2CDevice<'static, S>,
// 			I2CDevice::new(self.i2c_mux, self.address)
// 		);

// 		let i2c_syscalls = static_init_half!(
// 			static_buffer.1,
// 			I2CMasterDriver<'static, I2CDevice<'static, S>>,
// 			I2CMasterDriver::new(syscall_i2c_device)
// 		);

// 		static mut I2C_READ_BUF: [u8; 255] = [0; 255];
// 		static mut I2C_WRITE_BUF: [u8; 255] = [0; 255];

// 		i2c_syscalls.config_buffers(&mut I2C_READ_BUF, &mut I2C_WRITE_BUF);
// 		syscall_i2c_device.set_client(i2c_syscalls);

// 		i2c_syscalls
// 	}
// }

// impl<S: 'static + i2c::I2CMaster> I2CComponent<S> {
// 	pub fn new(mux: &'static MuxI2C<'static, S>, address: u8) -> Self {
// 		I2CComponent {
// 			i2c_mux: mux,
// 			address: address,
// 		}
// 	}
// }

// impl<S: 'static + i2c::I2CMaster> Component for I2CComponent<S> {
// 	type StaticInput = &'static mut MaybeUninit<I2CDevice<'static>>;
// 	type Output = &'static I2CDevice<'static>;

// 	unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
// 		let i2c_device = static_init_half!(
// 			static_buffer,
// 			I2CDevice<'static>,
// 			I2CDevice::new(self.i2c_mux, self.address)
// 		);

// 		i2c_device
// 	}
// }
