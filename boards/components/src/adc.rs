use core::mem::MaybeUninit;

// use capsules::adc::Adc;
use capsules::virtual_adc::{AdcUser, MuxAdc};
use kernel::component::Component;
use kernel::hil::adc;
use kernel::static_init_half;

#[macro_export]
macro_rules! adc_mux_component_helper {
    ($A:ty) => {{
        use capsules::virtual_adc::MuxAdc;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<MuxAdc<'static, $A>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

#[macro_export]
macro_rules! adc_component_helper {
    ($A:ty) => {{
        use capsules::virtual_adc::AdcUser;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<AdcUser<'static, $A>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

pub struct AdcMuxComponent<A: 'static + adc::Adc> {
    adc: &'static A,
}

pub struct AdcComponent<A: 'static + adc::Adc> {
    adc_mux: &'static MuxAdc<'static, A>,
    channel: A::Channel,
}

impl<A: 'static + adc::Adc> AdcMuxComponent<A> {
    pub fn new(adc: &'static A) -> Self {
        AdcMuxComponent { adc: adc }
    }
}

impl<A: 'static + adc::Adc> Component for AdcMuxComponent<A> {
    type StaticInput = &'static mut MaybeUninit<MuxAdc<'static, A>>;
    type Output = &'static MuxAdc<'static, A>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let adc_mux = static_init_half!(static_buffer, MuxAdc<'static, A>, MuxAdc::new(self.adc));

        self.adc.set_client(adc_mux);

        adc_mux
    }
}

impl<A: 'static + adc::Adc> AdcComponent<A> {
    pub fn new(mux: &'static MuxAdc<'static, A>, channel: A::Channel) -> Self {
        AdcComponent {
            adc_mux: mux,
            channel: channel,
        }
    }
}

impl<A: 'static + adc::Adc> Component for AdcComponent<A> {
    type StaticInput = &'static mut MaybeUninit<AdcUser<'static, A>>;
    type Output = &'static AdcUser<'static, A>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let adc_device = static_init_half!(
            static_buffer,
            AdcUser<'static, A>,
            AdcUser::new(self.adc_mux, self.channel)
        );

        adc_device
    }
}
