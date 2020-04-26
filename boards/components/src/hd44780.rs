//! Components for the HD447880 LCD controller.
//!
//! Usage
//! -----
//! ```rust
//! let lcd = components::hd44780::HD44780Component::new(board_kernel).finalize(
//!     components::hd44780_component_helper!(
//!         // timer
//!         stm32f4xx::tim2::Tim2,
//!         // alarm mux
//!         mux_alarm
//!         // rs pin
//!         stm32f4xx::gpio::PinId::PF13.get_pin().as_ref().unwrap(),
//!         // en pin
//!         stm32f4xx::gpio::PinId::PE11.get_pin().as_ref().unwrap(),
//!         // data 4 pin
//!         stm32f4xx::gpio::PinId::PF14.get_pin().as_ref().unwrap(),
//!         // data 5 pin
//!         stm32f4xx::gpio::PinId::PE13.get_pin().as_ref().unwrap(),
//!         // data 6 pin
//!         stm32f4xx::gpio::PinId::PF15.get_pin().as_ref().unwrap(),
//!         // data 7 pin
//!         stm32f4xx::gpio::PinId::PG14.get_pin().as_ref().unwrap()
//!     )
//! );
//! ```
use capsules::hd44780::HD44780;
use capsules::virtual_alarm::VirtualMuxAlarm;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::time;
use kernel::hil::time::Alarm;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! hd44780_component_helper {
    ($A:ty, $alarm_mux: expr, $rs:expr, $en: expr, $data_4_pin: expr, $data_5_pin: expr, $data_6_pin: expr, $data_7_pin: expr) => {{
        use capsules::hd44780::HD44780;
        use core::mem::MaybeUninit;
        use kernel::static_init;
        static hd44780_alarm: VirtualMuxAlarm<'static, $A> = static_init!(
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new($alarm_mux)
        );
        static mut BUF: MaybeUninit<HD44780<'static, VirtualMuxAlarm<'static, $A>>> =
            MaybeUninit::uninit();
        (
            &hd44780_alarm,
            &mut BUF,
            $rs,
            $en,
            $data_4_pin,
            $data_5_pin,
            $data_6_pin,
            $data_7_pin,
        )
    };};
}

pub struct HD44780Component<A: 'static + time::Alarm<'static>> {
    board_kernel: &'static kernel::Kernel,
    _alarm: PhantomData<A>,
}

impl<A: 'static + time::Alarm<'static>> HD44780Component<A> {
    pub fn new(board_kernel: &'static kernel::Kernel) -> HD44780Component<A> {
        HD44780Component {
            board_kernel: board_kernel,
            _alarm: PhantomData,
        }
    }
}

impl<A: 'static + time::Alarm<'static>> Component for HD44780Component<A> {
    type StaticInput = (
        &'static VirtualMuxAlarm<'static, A>,
        &'static mut MaybeUninit<HD44780<'static, VirtualMuxAlarm<'static, A>>>,
        &'static dyn kernel::hil::gpio::Pin,
        &'static dyn kernel::hil::gpio::Pin,
        &'static dyn kernel::hil::gpio::Pin,
        &'static dyn kernel::hil::gpio::Pin,
        &'static dyn kernel::hil::gpio::Pin,
        &'static dyn kernel::hil::gpio::Pin,
    );
    type Output = &'static HD44780<'static, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_lcd = self.board_kernel.create_grant(&grant_cap);

        let hd44780 = static_init_half!(
            static_buffer.1,
            capsules::hd44780::HD44780<'static, VirtualMuxAlarm<'static, A>>,
            capsules::hd44780::HD44780::new(
                static_buffer.2,
                static_buffer.3,
                static_buffer.4,
                static_buffer.5,
                static_buffer.6,
                static_buffer.7,
                &mut capsules::hd44780::BUFFER,
                &mut capsules::hd44780::ROW_OFFSETS,
                static_buffer.0,
                grant_lcd,
            )
        );
        static_buffer.0.set_client(hd44780);

        hd44780
    }
}
