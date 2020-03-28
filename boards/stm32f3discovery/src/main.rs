//! Board file for Nucleo-F446RE development board
//!
//! - <https://www.st.com/en/evaluation-tools/nucleo-f446re.html>

#![no_std]
#![no_main]
#![feature(asm, core_intrinsics)]
#![deny(missing_docs)]

use capsules::virtual_alarm::VirtualMuxAlarm;
use components::gpio::GpioComponent;
use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil::gpio::Configure;
use kernel::hil::gpio::Output;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};

// Setup
mod setup;

/// Support routines for debugging I/O.
pub mod io;

// Unit Tests for drivers.
#[allow(dead_code)]
mod virtual_uart_rx_test;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

// Actual memory for holding the active process structures.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None, None, None, None];

// Static reference to chip for panic dumps.
static mut CHIP: Option<&'static stm32f3xx::chip::Stm32f3xx> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 32768] = [0; 32768];

// Force the emission of the `.apps` segment in the kernel elf image
// NOTE: This will cause the kernel to overwrite any existing apps when flashed!
#[used]
#[link_section = ".app.hack"]
static APP_HACK: u8 = 0;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// A structure representing this platform that holds references to all
/// capsules for this platform.
struct STM32F3Discovery {
    console: &'static capsules::console::Console<'static>,
    ipc: kernel::ipc::IPC,
    gpio: &'static capsules::gpio::GPIO<'static>,
    led: &'static capsules::led::LED<'static>,
    button: &'static capsules::button::Button<'static>,
    alarm: &'static capsules::alarm::AlarmDriver<
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

/// Helper function called during bring-up that configures multiplexed I/O.
unsafe fn set_pin_primary_functions() {
    use stm32f3xx::exti::{LineId, EXTI};
    use stm32f3xx::gpio::{AlternateFunction, Mode, PinId, PortId, PORT};
    use stm32f3xx::syscfg::SYSCFG;

    SYSCFG.enable_clock();

    PORT[PortId::A as usize].enable_clock();
    PORT[PortId::B as usize].enable_clock();
    PORT[PortId::C as usize].enable_clock();
    PORT[PortId::D as usize].enable_clock();
    PORT[PortId::E as usize].enable_clock();
    PORT[PortId::F as usize].enable_clock();

    PinId::PE14.get_pin().as_ref().map(|pin| {
        pin.make_output();
        pin.set();
    });

    // User LD3 is connected to PE09. Configure PE09 as `debug_gpio!(0, ...)`
    PinId::PE09.get_pin().as_ref().map(|pin| {
        pin.make_output();

        // Configure kernel debug gpios as early as possible
        kernel::debug::assign_gpios(Some(pin), None, None);
    });

    // // pc4 and pc5 (USART1) is connected to ST-LINK virtual COM port
    PinId::PC04.get_pin().as_ref().map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART1_TX
        pin.set_alternate_function(AlternateFunction::AF7);
    });
    PinId::PA05.get_pin().as_ref().map(|pin| {
        pin.set_mode(Mode::AlternateFunctionMode);
        // AF7 is USART1_RX
        pin.set_alternate_function(AlternateFunction::AF7);
    });

    // button is connected on pa00
    PinId::PA00.get_pin().as_ref().map(|pin| {
        // By default, upon reset, the pin is in input mode, with no internal
        // pull-up, no internal pull-down (i.e., floating).
        //
        // Only set the mapping between EXTI line and the Pin and let capsule do
        // the rest.
        EXTI.associate_line_gpiopin(LineId::Exti0, pin);
    });
    cortexm4::nvic::Nvic::new(stm32f3xx::nvic::EXTI0).enable();
}

/// Helper function for miscellaneous peripheral functions
unsafe fn setup_peripherals() {
    use stm32f3xx::tim2::TIM2;

    // USART1 IRQn is 37
    cortexm4::nvic::Nvic::new(stm32f3xx::nvic::USART1).enable();

    // TIM2 IRQn is 28
    TIM2.enable_clock();
    TIM2.start();
    cortexm4::nvic::Nvic::new(stm32f3xx::nvic::TIM2).enable();
}

/// Reset Handler.
///
/// This symbol is loaded into vector table by the STM32F303VCT6 chip crate.
/// When the chip first powers on or later does a hard reset, after the core
/// initializes all the hardware, the address of this function is loaded and
/// execution begins here.
#[no_mangle]
pub unsafe fn reset_handler() {
    stm32f3xx::init();

    // We use the default HSI 8Mhz clock

    set_pin_primary_functions();

    setup_peripherals();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));
    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    let chip = static_init!(
        stm32f3xx::chip::Stm32f3xx,
        stm32f3xx::chip::Stm32f3xx::new()
    );
    CHIP = Some(chip);

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);

    let stm32f3discovery = setup::init(&board_kernel, &dynamic_deferred_caller, &process_management_capability, &memory_allocation_capability);

    // // Optional kernel tests
    // //
    // // See comment in `boards/imix/src/main.rs`
    // virtual_uart_rx_test::run_virtual_uart_receive(mux_uart);

    // hprintln!("Initialization complete. Entering main loop").unwrap ();
    debug!("Initialization complete. Entering main loop");

    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;
    }

    kernel::procs::load_processes(
        board_kernel,
        chip,
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_management_capability,
    );

    board_kernel.kernel_loop(
        &stm32f3discovery,
        chip,
        Some(&stm32f3discovery.ipc),
        &main_loop_capability,
    );
}
