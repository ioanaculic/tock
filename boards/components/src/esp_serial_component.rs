//! Components for Console, the generic serial interface, and for multiplexed access
//! to UART.
//!
//!
//! This provides two Components, `ConsoleComponent`, which implements a buffered
//! read/write console over a serial port, and `UartMuxComponent`, which provides
//! multiplexed access to hardware UART. As an example, the serial port used for
//! console on Imix is typically USART3 (the DEBUG USB connector).
//!
//! Usage
//! -----
//! ```rust
//! let uart_mux = UartMuxComponent::new(&sam4l::usart::USART3,
//!                                      115200,
//!                                      deferred_caller).finalize(());
//! let console = ConsoleComponent::new(board_kernel, uart_mux).finalize(());
//! ```
// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 1/08/2020

use capsules::esp_serial;
use capsules::esp_serial::Link;
use capsules::virtual_uart::{MuxUart, UartDevice};
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::static_init;

// pub struct UartMuxComponent {
//     uart: &'static dyn uart::Uart<'static>,
//     baud_rate: u32,
//     deferred_caller: &'static DynamicDeferredCall,
// }

// impl UartMuxComponent {
//     pub fn new(
//         uart: &'static dyn uart::Uart<'static>,
//         baud_rate: u32,
//         deferred_caller: &'static DynamicDeferredCall,
//     ) -> UartMuxComponent {
//         UartMuxComponent {
//             uart,
//             baud_rate,
//             deferred_caller,
//         }
//     }
// }

// impl Component for UartMuxComponent {
//     type StaticInput = ();
//     type Output = &'static MuxUart<'static>;

//     unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
//         let uart_mux = static_init!(
//             MuxUart<'static>,
//             MuxUart::new(
//                 self.uart,
//                 &mut capsules::virtual_uart::RX_BUF,
//                 self.baud_rate,
//                 self.deferred_caller,
//             )
//         );
//         uart_mux.initialize_callback_handle(
//             self.deferred_caller
//                 .register(uart_mux)
//                 .expect("no deferred call slot available for uart mux"),
//         );

//         uart_mux.initialize();
//         hil::uart::Transmit::set_transmit_client(self.uart, uart_mux);
//         hil::uart::Receive::set_receive_client(self.uart, uart_mux);

//         uart_mux
//     }
// }

pub struct ESPSerialComponent {
    board_kernel: &'static kernel::Kernel,
    uart_mux: &'static MuxUart<'static>,
}

impl ESPSerialComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        uart_mux: &'static MuxUart,
    ) -> ESPSerialComponent {
        ESPSerialComponent {
            board_kernel: board_kernel,
            uart_mux: uart_mux,
        }
    }
}

impl Component for ESPSerialComponent {
    type StaticInput = ();
    type Output = &'static esp_serial::EspSerial<'static>;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        // Create virtual device for console.
        let console_uart = static_init!(UartDevice, UartDevice::new(self.uart_mux, true));
        console_uart.setup();
        let link0 = static_init!(Link, Link::new(0));
        let link1 = static_init!(Link, Link::new(1));
        let link2 = static_init!(Link, Link::new(2));
        let link3 = static_init!(Link, Link::new(3));
        let link4 = static_init!(Link, Link::new(4));
        let console = static_init!(
            esp_serial::EspSerial<'static>,
            esp_serial::EspSerial::new(
                console_uart,
                &mut esp_serial::WRITE_BUF,
                &mut esp_serial::READ_BUF,
                &mut esp_serial::HELPER_BUF,
                [link0, link1, link2, link3, link4],
                self.board_kernel.create_grant(&grant_cap)
            )
        );
        hil::uart::Transmit::set_transmit_client(console_uart, console);
        hil::uart::Receive::set_receive_client(console_uart, console);

        console
    }
}
