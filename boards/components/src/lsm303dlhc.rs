//! Components for the LSM303DLHC sensor.
//!
//! I2C Interface
//!
//! Usage
//! -----
//! ```rust
//! let lsm303dlhc = components::lsm303dlhc::Lsm303dlhcI2CComponent::new()
//!    .finalize(components::lsm303dlhc_i2c_component_helper!(mux_i2c));
//!
//! lsm303dlhc.configure(
//!    lsm303dlhc::Lsm303dlhcAccelDataRate::DataRate25Hz,
//!    false,
//!    lsm303dlhc::Lsm303dlhcScale::Scale2G,
//!    false,
//!    true,
//!    lsm303dlhc::Lsm303dlhcMagnetoDataRate::DataRate3_0Hz,
//!    lsm303dlhc::Lsm303dlhcRange::Range4_7G,
//! );
//! ```
use capsules::lsm303dlhc::Lsm303dlhcI2C;
use capsules::virtual_i2c::I2CDevice;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::static_init_half;
use core::marker::PhantomData;
use kernel::hil::i2c;

// Setup static space for the objects.
#[macro_export]
macro_rules! lsm303dlhc_i2c_component_helper {
    ($I:ty, $S:ty, $i2c_mux: expr) => {{
        use capsules::lsm303dlhc::Lsm303dlhcI2C;
        use capsules::virtual_i2c::I2CDevice;
        use core::mem::MaybeUninit;
        let accelerometer_i2c = components::i2c::I2CComponent::new($i2c_mux, 0x19)
            .finalize(components::i2c_component_helper!($I, $S));
        let magnetometer_i2c = components::i2c::I2CComponent::new($i2c_mux, 0x1e)
            .finalize(components::i2c_component_helper!($I, $S));
        static mut lsm303dlhc: MaybeUninit<Lsm303dlhcI2C<'static, I2CDevice<'static, $I, $S>>> = MaybeUninit::uninit();
        (&accelerometer_i2c, &magnetometer_i2c, &mut lsm303dlhc)
    };};
}

pub struct Lsm303dlhcI2CComponent<I: 'static + i2c::I2CMaster, S: 'static + i2c::SMBusMaster> {
    _i2c: PhantomData<I>,
    _smbus: PhantomData<S>
}

impl<I: i2c::I2CMaster, S: i2c::SMBusMaster> Lsm303dlhcI2CComponent<I, S> {
    pub fn new() -> Lsm303dlhcI2CComponent<I, S> {
        Lsm303dlhcI2CComponent {
            _i2c: PhantomData,
            _smbus: PhantomData
        }
    }
}

impl<I: i2c::I2CMaster, S: i2c::SMBusMaster> Component for Lsm303dlhcI2CComponent<I, S> {
    type StaticInput = (
        &'static I2CDevice<'static, I, S>,
        &'static I2CDevice<'static, I, S>,
        &'static mut MaybeUninit<Lsm303dlhcI2C<'static, I2CDevice<'static, I, S>>>,
    );
    type Output = &'static Lsm303dlhcI2C<'static, I2CDevice<'static, I, S>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let lsm303dlhc = static_init_half!(
            static_buffer.2,
            Lsm303dlhcI2C<'static, I2CDevice<'static, I, S>>,
            Lsm303dlhcI2C::new(
                static_buffer.0,
                static_buffer.1,
                &mut capsules::lsm303dlhc::BUFFER
            )
        );
        static_buffer.0.set_client(lsm303dlhc);
        static_buffer.1.set_client(lsm303dlhc);

        lsm303dlhc
    }
}
