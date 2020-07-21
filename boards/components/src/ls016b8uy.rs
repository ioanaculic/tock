//! Components for the LS016B8UY SPI screen.
//!
//! SPI Interface
//!
//! Usage
//! -----
//! ```rust
//! let tft = components::ls016b8uy::LS0168BUYComponent::new(alarm_mux).finalize(
//!     components::ls016b8u6_component_helper!(
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
use capsules::ls016b8uy::LS016B8UY;
use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::memory_async::Memory;
use kernel::hil::time;
use kernel::hil::time::Alarm;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! ls016b8u6_component_helper {
    ($fsmc:expr, $A:ty, $reset:expr) => {{
        use capsules::ls016b8uy::LS016B8UY;
        use capsules::virtual_alarm::VirtualMuxAlarm;
        use capsules::virtual_spi::VirtualSpiMasterDevice;
        use core::mem::MaybeUninit;
        use kernel::hil::memory_async::Memory;
        use kernel::hil::spi::{self, SpiMasterDevice};
        let ls0168buy_mem: &'static dyn Memory = $fsmc;
        static mut ls0168buy_alarm: MaybeUninit<VirtualMuxAlarm<'static, $A>> =
            MaybeUninit::uninit();
        static mut ls016b8uy: MaybeUninit<LS016B8UY<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        (ls0168buy_mem, &mut ls0168buy_alarm, $reset, &mut ls016b8uy)
    };};
}

pub struct LS0168BUYComponent<A: 'static + time::Alarm<'static>> {
    alarm_mux: &'static MuxAlarm<'static, A>,
}

impl<A: 'static + time::Alarm<'static>> LS0168BUYComponent<A> {
    pub fn new(alarm_mux: &'static MuxAlarm<'static, A>) -> LS0168BUYComponent<A> {
        LS0168BUYComponent {
            alarm_mux: alarm_mux,
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for LS0168BUYComponent<A> {
    type StaticInput = (
        &'static dyn Memory,
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static dyn kernel::hil::gpio::Pin,
        &'static mut MaybeUninit<LS016B8UY<'static, VirtualMuxAlarm<'static, A>>>,
    );
    type Output = &'static LS016B8UY<'static, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let ls0168buy_alarm = static_init_half!(
            static_buffer.1,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );

        let ls016b8uy = static_init_half!(
            static_buffer.3,
            LS016B8UY<'static, VirtualMuxAlarm<'static, A>>,
            LS016B8UY::new(
                static_buffer.0,
                ls0168buy_alarm,
                static_buffer.2,
                &mut capsules::ls016b8uy::BUFFER,
                &mut capsules::ls016b8uy::SEQUENCE_BUFFER
            )
        );
        static_buffer.0.set_client(ls016b8uy);
        ls0168buy_alarm.set_client(ls016b8uy);

        ls016b8uy
    }
}
