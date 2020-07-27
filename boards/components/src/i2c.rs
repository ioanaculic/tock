//! Components for I2C.
//!
//! This provides two components.
//!
//! 1. `I2CMuxComponent` provides a virtualization layer for a I2C bus.
//!
//! 2. `I2CComponent` provides a virtualized client to the I2C bus.
//!
//! Usage
//! -----
//! ```rust
//! let mux_i2c = components::i2c::I2CMuxComponent::new(&stm32f3xx::i2c::I2C1).finalize(components::i2c_mux_component_helper!());
//! let client_i2c = components::i2c::I2CComponent::new(mux_i2c, 0x19).finalize(components::i2c_component_helper!());
//! ```

// Author: Alexandru Radovici <msg4alex@gmail.com>

use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use core::mem::MaybeUninit;
use kernel::common::dynamic_deferred_call::DynamicDeferredCall;
use kernel::component::Component;
use kernel::hil::i2c;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! i2c_mux_component_helper {
    ($I: ty, $S: ty) => {{
        use capsules::virtual_i2c::MuxI2C;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<MuxI2C<'static, $I, $S>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

#[macro_export]
macro_rules! i2c_component_helper {
    ($I: ty, $S: ty) => {{
        use capsules::virtual_i2c::I2CDevice;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<I2CDevice<'static, $I, $S>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

pub struct I2CMuxComponent<I: 'static + i2c::I2CMaster, S: 'static + i2c::SMBusMaster> {
    i2c: &'static I,
    smbus: Option<&'static S>,
    deferred_caller: &'static DynamicDeferredCall,
}

pub struct I2CComponent<I: 'static + i2c::I2CMaster, S: 'static + i2c::SMBusMaster> {
    i2c_mux: &'static MuxI2C<'static, I, S>,
    address: u8,
}

impl<I: i2c::I2CMaster, S: i2c::SMBusMaster> I2CMuxComponent<I, S> {
    pub fn new(
        i2c: &'static I,
        smbus: Option<&'static S>,
        deferred_caller: &'static DynamicDeferredCall,
    ) -> Self {
        I2CMuxComponent {
            i2c,
            smbus,
            deferred_caller,
        }
    }
}

impl<I: 'static + i2c::I2CMaster, S: 'static + i2c::SMBusMaster> Component for I2CMuxComponent<I, S> {
    type StaticInput = &'static mut MaybeUninit<MuxI2C<'static, I, S>>;
    type Output = &'static MuxI2C<'static, I, S>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let mux_i2c = static_init_half!(
            static_buffer,
            MuxI2C<'static, I, S>,
            MuxI2C::new(self.i2c, self.smbus, self.deferred_caller)
        );

        mux_i2c.initialize_callback_handle(
            self.deferred_caller
                .register(mux_i2c)
                .expect("no deferred call slot available for I2C mux"),
        );

        self.i2c.set_master_client(mux_i2c);

        mux_i2c
    }
}

impl<I: i2c::I2CMaster, S: i2c::SMBusMaster> I2CComponent<I, S> {
    pub fn new(mux: &'static MuxI2C<'static, I, S>, address: u8) -> Self {
        I2CComponent {
            i2c_mux: mux,
            address: address,
        }
    }
}

impl<I: 'static + i2c::I2CMaster, S: 'static + i2c::SMBusMaster> Component for I2CComponent<I,S> {
    type StaticInput = &'static mut MaybeUninit<I2CDevice<'static, I, S>>;
    type Output = &'static I2CDevice<'static, I, S>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let i2c_device = static_init_half!(
            static_buffer,
            I2CDevice<'static, I, S>,
            I2CDevice::new(self.i2c_mux, self.address)
        );

        i2c_device
    }
}
