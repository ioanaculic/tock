use capsules::virtual_alarm::VirtualMuxAlarm;
use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil::gpio::Configure;
use kernel::hil::gpio::Output;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};

/// Support routines for debugging I/O.
#[path = "io.rs"]
mod io;

/// A structure representing this platform that holds references to all
/// capsules for this platform.
pub struct STM32F3Discovery {
    pub console: &'static capsules::console::Console<'static>,
    pub ipc: kernel::ipc::IPC,
    pub gpio: &'static capsules::gpio::GPIO<'static>,
    pub led: &'static capsules::led::LED<'static>,
    pub button: &'static capsules::button::Button<'static>,
    pub alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, stm32f3xx::tim2::Tim2<'static>>,
    >,
}


/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for STM32F3Discovery {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
            capsules::button::DRIVER_NUM => f(Some(self.button)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::gpio::DRIVER_NUM => f(Some(self.gpio)),
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

pub fn init (board_kernel, dynamic_deferred_caller, process_management_capability, memory_allocation_capability) -> STM32F3Discovery{
    // UART
    // Create a shared UART channel for kernel debug.
    stm32f3xx::usart::USART1.enable_clock();
    let uart_mux = components::console::UartMuxComponent::new(
        &stm32f3xx::usart::USART1,
        115200,
        dynamic_deferred_caller,
    )
    .finalize(());

    // `finalize()` configures the underlying USART, so we need to
    // tell `send_byte()` not to configure the USART again.
    io::WRITER.set_initialized();

    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    let led = components::led::LedsComponent::new().finalize(components::led_component_helper!(
        (
            stm32f3xx::gpio::PinId::PE09.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f3xx::gpio::PinId::PE08.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f3xx::gpio::PinId::PE10.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f3xx::gpio::PinId::PE15.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f3xx::gpio::PinId::PE11.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f3xx::gpio::PinId::PE14.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f3xx::gpio::PinId::PE12.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        ),
        (
            stm32f3xx::gpio::PinId::PE13.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveHigh
        )
    ));

    // BUTTONs
    let button = components::button::ButtonComponent::new(board_kernel).finalize(
        components::button_component_helper!((
            stm32f3xx::gpio::PinId::PA00.get_pin().as_ref().unwrap(),
            kernel::hil::gpio::ActivationMode::ActiveLow,
            kernel::hil::gpio::FloatingState::PullNone
        )),
    );

    // ALARM

    let tim2 = &stm32f3xx::tim2::TIM2;
    let mux_alarm = components::alarm::AlarmMuxComponent::new(tim2).finalize(
        components::alarm_mux_component_helper!(stm32f3xx::tim2::Tim2),
    );

    let alarm = components::alarm::AlarmDriverComponent::new(board_kernel, mux_alarm)
        .finalize(components::alarm_component_helper!(stm32f3xx::tim2::Tim2));

    // GPIO
    let gpio = GpioComponent::new(board_kernel).finalize(components::gpio_component_helper!(
        // Left outer connector
        stm32f3xx::gpio::PinId::PC01.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC03.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA01.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA03.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PF04.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA05.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA07.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC05.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB01.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE07.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE09.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE11.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE13.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB11.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB13.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB15.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD09.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD11.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD13.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD15.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC06.get_pin().as_ref().unwrap(),
        // Left inner connector
        stm32f3xx::gpio::PinId::PC00.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC02.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PF02.get_pin().as_ref().unwrap(),
        // stm32f3xx::gpio::PinId::PA00.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA02.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA04.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA06.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC04.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB00.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB02.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE08.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE10.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE12.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE14.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB10.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB12.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB14.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD08.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD10.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD14.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC07.get_pin().as_ref().unwrap(),
        // Right inner connector
        stm32f3xx::gpio::PinId::PF09.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PF00.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC14.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE06.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE04.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE02.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB08.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB06.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB04.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD07.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD05.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD03.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC12.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC10.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA14.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PF06.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA12.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA10.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA08.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC08.get_pin().as_ref().unwrap(),
        // Right outer connector
        stm32f3xx::gpio::PinId::PF10.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PF01.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC15.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC13.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE05.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PE03.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB09.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB07.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB05.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PB03.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD06.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD04.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PD02.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC11.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA15.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA13.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA11.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PA09.get_pin().as_ref().unwrap(),
        stm32f3xx::gpio::PinId::PC09.get_pin().as_ref().unwrap()
    ));

    let stm32f3discovery = STM32F3Discovery {
        console: console,
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
        gpio: gpio,
        led: led,
        button: button,
        alarm: alarm,
    };

    stm32f3discovery
}