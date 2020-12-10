//! Component for the BMP280 sensor.
//!
//! I2C Interface
//!
//! Usage
//! -----
//!
//! With the default i2c address
//! ```rust
//! let bmp280 = components::bmp280::BMP280Component::new(sensors_i2c_bus).finalize(
//!         components::sht3x_component_helper!(nrf52::rtc::Rtc<'static>),
//!     );
//! bmp280.reset();
//! ```
//!
//! With a specified i2c address
//! ```rust
//! let bmp280 = components::bmp280::BMP280Component::new(sensors_i2c_bus).finalize(
//!         components::sht3x_component_helper!(nrf52::rtc::Rtc<'static>, capsules::bmp280::BASE_ADDR << 1),
//!     );
//! bmp280.reset();
//! ```

use capsules::bmp280::BMP280;
use capsules::virtual_i2c::MuxI2C;
use core::mem::MaybeUninit;
use kernel::component::Component;

use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! sht3x_component_helper {
    ($A:ty) => {{
        use capsules::bmp280;
        $crate::sht3x_component_helper!($A, bmp280::BASE_ADDR)
    }};

    // used for specifically stating the i2c address
    // as some boards (like nrf52) require a shift
    ($A:ty, $address: expr) => {{
        use capsules::bmp280::BMP280;
        use capsules::virtual_i2c::I2CDevice;
        use core::mem::MaybeUninit;

        static mut BUFFER: [u8; 6] = [0; 6];

        static mut bmp280: MaybeUninit<BMP280<'static>> = MaybeUninit::uninit();
        (&mut BUFFER, &mut bmp280, $address)
    }};
}

pub struct BMP280Component {
    i2c_mux: &'static MuxI2C<'static>,
}

impl BMP280Component {
    pub fn new(i2c_mux: &'static MuxI2C<'static>) -> BMP280Component {
        BMP280Component { i2c_mux }
    }
}

impl Component for BMP280Component {
    type StaticInput = (
        &'static mut [u8],
        &'static mut MaybeUninit<BMP280<'static>>,
        u8,
    );
    type Output = &'static BMP280<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let bmp280_i2c = crate::i2c::I2CComponent::new(self.i2c_mux, static_buffer.2)
            .finalize(crate::i2c_component_helper!());

        let bmp280 = static_init_half!(
            static_buffer.1,
            BMP280<'static>,
            BMP280::new(bmp280_i2c, static_buffer.0)
        );

        bmp280_i2c.set_client(bmp280);

        bmp280
    }
}
