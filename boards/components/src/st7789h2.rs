//! Components for the ST7789H2 SPI screen.
//!
//! SPI Interface
//!
//! Usage
//! -----
//! ```rust
//! let tft = components::st7789h2::ST7789H2Component::new(alarm_mux).finalize(
//!     components::st7789h2_component_helper!(
//!         // spi type
//!         stm32f4xx::spi::Spi,
//!         // chip select
//!         stm32f4xx::gpio::PinId::PE03,
//!         // spi mux
//!         spi_mux,
//!         // timer type
//!         stm32f4xx::tim2::Tim2,
//!         // dc pin
//!         stm32f4xx::gpio::PinId::PA00.get_pin().as_ref().unwrap(),
//!         // reset pin
//!         stm32f4xx::gpio::PinId::PA00.get_pin().as_ref().unwrap()
//!     )
//! );
//! ```
use capsules::st7789h2::ST7789H2;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::memory_async::Memory;
use kernel::hil::time;
use kernel::hil::time::Alarm;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! st7789h2_component_helper {
    ($fsmc:expr, $A:ty, $reset:expr) => {{
        use capsules::st7789h2::ST7789H2;
        use capsules::virtual_alarm::VirtualMuxAlarm;
        use capsules::virtual_spi::VirtualSpiMasterDevice;
        use core::mem::MaybeUninit;
        use kernel::hil::memory_async::Memory;
        use kernel::hil::spi::{self, SpiMasterDevice};
        let ls0168buy_mem: &'static dyn Memory = $fsmc;
        static mut st7789h2_alarm: MaybeUninit<VirtualMuxAlarm<'static, $A>> =
            MaybeUninit::uninit();
        static mut st7789h2: MaybeUninit<ST7789H2<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        (ls0168buy_mem, &mut st7789h2_alarm, $reset, &mut st7789h2)
    };};
}

pub struct ST7789H2Component<A: 'static + time::Alarm<'static>> {
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<A: 'static + time::Alarm<'static>> ST7789H2Component<A> {
    pub fn new(alarm_mux: &'static MuxAlarm<'static, A>) -> ST7789H2Component<A> {
        ST7789H2Component {
            alarm_mux: alarm_mux,
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for ST7789H2Component<A> {
    type StaticInput = (
        &'static dyn Memory,
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static dyn kernel::hil::gpio::Pin,
        &'static mut MaybeUninit<ST7789H2<'static, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static ST7789H2<'static, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let st7789h2_alarm = static_init_half!(
            static_buffer.1,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );

        let st7789h2 = static_init_half!(
            static_buffer.3,
            ST7789H2<'static, VirtualMuxAlarm<'static, A>>,
            ST7789H2::new(
                static_buffer.0,
                st7789h2_alarm,
                static_buffer.2,
                &mut capsules::st7789h2::BUFFER,
                &mut capsules::st7789h2::SEQUENCE_BUFFER
            )
        );
        static_buffer.0.set_client(st7789h2);
        st7789h2_alarm.set_client(st7789h2);

        st7789h2
    }
}
