use capsules::temperature_stm::TemperatureSTM;
use capsules::virtual_adc::AdcUser;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::adc;
use kernel::static_init_half;

#[macro_export]
macro_rules! temperaturestm_adc_component_helper {
    ($A:ty, $select: expr, $spi_mux: expr) => {{
        use capsules::temperature_stm::TemperatureSTM;
        use capsules::virtual_adc::AdcUser;
        use core::mem::MaybeUninit;
        let mut temperature_stm_adc: &'static capsules::virtual_adc::AdcUser<'static, $A> =
            components::adc::AdcMuxComponent::new($adc_mux)
                .finalize(components::adc_component_helper!($A));
        static mut temperature_stm: MaybeUninit<TemperatureSTM<'static>> = MaybeUninit::uninit();
        (&mut temperature_stm_adc, &mut temperature_stm)
    };};
}

pub struct TemperatureSTMAdcComponent<A: 'static + adc::AdcChannel> {
    _select: PhantomData<A>,
}

impl<A: 'static + adc::AdcChannel> TemperatureSTMAdcComponent<S> {
    pub fn new() -> TemperatureSTMAdcComponent<S> {
        TemperatureSTMAdcComponent {
            _select: PhantomData,
        }
    }
}

impl<A: 'static + adc::AdcChannel> Component for TemperatureSTMAdcComponent<S> {
    type StaticInput = (
        &'static AdcUser<'static, S>,
        &'static mut MaybeUninit<TemperatureSTM<'static>>,
    );
    type Output = &'static TemperatureSTM<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let temperature_stm = static_init_half!(
            static_buffer.1,
            TemperatureSTM<'static>,
            TemperatureSTM::new(
                static_buffer.0,
                // slope: f32,
                // v_25: f32
            );
        );
        static_buffer.0.set_client(temperature_stm);
        temperature_stm.configure();

        temperature_stm
    }
}
