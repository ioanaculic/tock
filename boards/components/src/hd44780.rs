//! Components for the HD447880 LCD controller.
//!
//! Usage
//! -----
//! ```rust
//! let lcd = components::hd44780::HD44780Component::new(mux_alarm).finalize(
//!     components::hd44780_component_helper!(
//!         stm32f429zi::tim2::Tim2,
//!         // rs pin
//!         stm32f429zi::gpio::PinId::PF13.get_pin().as_ref().unwrap(),
//!         // en pin
//!         stm32f429zi::gpio::PinId::PE11.get_pin().as_ref().unwrap(),
//!         // data 4 pin
//!         stm32f429zi::gpio::PinId::PF14.get_pin().as_ref().unwrap(),
//!         // data 5 pin
//!         stm32f429zi::gpio::PinId::PE13.get_pin().as_ref().unwrap(),
//!         // data 6 pin
//!         stm32f429zi::gpio::PinId::PF15.get_pin().as_ref().unwrap(),
//!         // data 7 pin
//!         stm32f429zi::gpio::PinId::PG14.get_pin().as_ref().unwrap()
//!     )
//! );
//! ```
use capsules::hd44780::{HD44780, HD44780Gpio};
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::time;
use kernel::hil::time::Alarm;
use kernel::static_init_half;
use core::marker::PhantomData;

// Setup static space for the objects.
#[macro_export]
macro_rules! hd44780_gpio_component_helper {
    ($rs:expr, $en: expr, $data1: expr, $data2: expr, $data3: expr, $data4: expr, $A:ty) => {{
        use capsules::hd44780::{HD44780, HD44780DirectGpio};
        use kernel::static_init_half;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<HD44780<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        static mut gpio_buffer: MaybeUninit<HD44780DirectGpio<'static>> = MaybeUninit::uninit();
        let gpio = static_init_half!(&mut gpio_buffer, HD44780DirectGpio<'static>, HD44780DirectGpio::new($rs, $en, $data1, $data2, $data3, $data4));
        (
            &mut BUF1,
            &mut BUF2,
            gpio,
        )
    };};
}

#[macro_export]
macro_rules! hd44780_i2c_component_helper {
    ($A:ty, $M:expr) => {{
        use capsules::hd44780::{HD44780, HD44780I2C};
        use capsules::mcp230xx::MCP230xx;
        use kernel::static_init_half;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<HD44780<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        static mut i2c_buffer: MaybeUninit<MCP230xx<'static>> = MaybeUninit::uninit();
        let mcp_i2c = static_init_half!(&mut i2c_buffer, HD44780I2C<'static>, I2C::new($M, &mut capsules::hd44780::MCP_PINS));
        (
            &mut BUF1,
            &mut BUF2,
            &mut mcp_i2c,
        )
    };};
}

pub struct HD44780Component<G: 'static + HD44780Gpio, A: 'static + time::Alarm<'static>> {
    alarm_mux: &'static MuxAlarm<'static, A>,
    _gpio: PhantomData<G>
}

impl<G: 'static + HD44780Gpio, A: 'static + time::Alarm<'static>> HD44780Component<G, A> {
    pub fn new(alarm_mux: &'static MuxAlarm<'static, A>) -> HD44780Component<G, A> {
        HD44780Component {
            alarm_mux: alarm_mux,
            _gpio: PhantomData,
        }
    }
}

impl<G: 'static + HD44780Gpio, A: 'static + time::Alarm<'static>> Component for HD44780Component<G, A> {
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<HD44780<'static, VirtualMuxAlarm<'static, A>>>,
        &'static G
    );
    type Output = &'static HD44780<'static, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let lcd_alarm = static_init_half!(
            static_buffer.0,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );

        let hd44780 = static_init_half!(
            static_buffer.1,
            capsules::hd44780::HD44780<'static, VirtualMuxAlarm<'static, A>>,
            capsules::hd44780::HD44780::new(
                static_buffer.2,
                &mut capsules::hd44780::ROW_OFFSETS,
                lcd_alarm,
            )
        );
        lcd_alarm.set_client(hd44780);

        hd44780
    }
}

// pub struct HD44780ComponentI2C<A: 'static + time::Alarm<'static>> {
//     alarm_mux: &'static MuxAlarm<'static, A>,
//     mcp_230xx: &'static MCP230xx<'static>,
// }

// impl<A: 'static + time::Alarm<'static>> HD44780ComponentI2C<A: 'static + time::Alarm<'static>> {
//     pub fn new(
//         alarm_mux: &'static MuxAlarm<'static, A>,
//         mcp_230xx: &'static MCP230xx<'static>,
//     ) -> HD44780ComponentI2C {
//         HD44780ComponentI2C {
//             alarm_mux: alarm_mux,
//             mcp_230xx: mcp_230xx,
//         }
//     }
// }

// impl<A: 'static + time::Alarm<'static>> Component for HD44780ComponentI2C<A: 'static + time::Alarm<'static>> {
//     type StaticInput = (
//         &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
//         &'static mut MaybeUninit<HD44780<'static, VirtualMuxAlarm<'static, A>>>,
//     );
//     type Output = ();
    
//     unsafe fn finalize
// }
