use capsules::virtual_pwm;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::pwm;
use kernel::static_init_half;

#[macro_export]
macro_rules! pwm_mux_component_helper {
    ($A:ty) => {{
        use capsules::virtual_pwm::MuxPwm;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<MuxPwm<'static, $A>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

#[macro_export]
macro_rules! pwm_component_helper {
    ($A:ty) => {{
        use capsules::virtual_pwm::PwmPinUser;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<virtual_pwm::PwmPinUser<'static, $A>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

pub struct PwmMuxComponent<P: 'static + pwm::Pwm> {
    pwm: &'static P,
}

pub struct PwmComponent<P: 'static + pwm::Pwm> {
    pwm_mux: &'static virtual_pwm::MuxPwm<'static, P>,
}

impl<P: 'static + pwm::Pwm> PwmMuxComponent<P> {
    pub fn new(pwm: &'static P) -> Self {
        PwmMuxComponent { pwm: pwm }
    }
}

pub struct PwmVirtualComponent {
    board_kernel: &'static kernel::Kernel,
}

impl<P: 'static + pwm::Pwm> Component for PwmMuxComponent<P> {
    type StaticInput = &'static mut MaybeUninit<virtual_pwm::MuxPwm<'static, P>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let pwm_mux = static_init_half!(static_buffer, virtual_pwm::MuxPwm<'static, P>);

        pwm_mux
    }
}

impl<P: 'static + pwm::Pwm> PwmComponent<P> {
    pub fn new(mux: &'static virtual_pwm::MuxPwm<'static, P>) -> Self {
        PwmComponent { pwm_mux: mux }
    }
}

impl<P: 'static + pwm::Pwm> Component for PwmComponent<P> {
    type StaticInput = &'static mut MaybeUninit<virtual_pwm::PwmPinUser<'static, P>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let pwm_device = static_init_half!(static_buffer, virtual_pwm::PwmPinUser<'static, A>,);

        pwm_device.add_to_mux();

        pwm_device
    }
}

impl PwmVirtualComponent {
    pub fn new(board_kernel: &'static kernel::Kernel) -> PwmVirtualComponent {
        PwmVirtualComponent {
            board_kernel: board_kernel,
        }
    }
}

impl Component for PwmVirtualComponent {
    type StaticInput = (&'static mut MaybeUninit<virtual_pwm::PwmPinUser<'static>>);
    type Output = &'static virtual_pwm::PwmPinUser<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let pwm = static_init_half!(static_buffer.0, virtual_pwm::PwmPinUser<'static>);

        pwm
    }
}
