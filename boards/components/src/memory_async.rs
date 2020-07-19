use capsules::memory_async::SpiMemory;
use capsules::virtual_spi::VirtualSpiMasterDevice;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::spi;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! spi_memory_component_helper {
    ($S:ty, $select:expr, $spi_mux: expr, $A:ty, $dc:expr, $reset:expr) => {{
        use capsules::memory_async::SpiMemory;
        use capsules::virtual_alarm::VirtualMuxAlarm;
        use capsules::virtual_spi::VirtualSpiMasterDevice;
        use core::mem::MaybeUninit;
        let mem_spi: &'static capsules::virtual_spi::VirtualSpiMasterDevice<'static, $S> =
            components::spi::SpiComponent::new($spi_mux, $select)
                .finalize(components::spi_component_helper!($S));
        static mut mem: MaybeUninit<SpiMemory<'static>> = MaybeUninit::uninit();
        (&mem_spi, &mut mem)
    };};
}

pub struct SpiMemoryComponent<S: 'static + spi::SpiMaster> {
    _select: PhantomData<S>,
}

impl<S: 'static + spi::SpiMaster> SpiMemoryComponent<S> {
    pub fn new() -> SpiMemoryComponent<S> {
        SpiMemoryComponent {
            _select: PhantomData,
        }
    }
}

impl<S: 'static + spi::SpiMaster> Component for SpiMemoryComponent<S> {
    type StaticInput = (
        &'static VirtualSpiMasterDevice<'static, S>,
        &'static mut MaybeUninit<SpiMemory<'static>>,
    );
    type Output = &'static SpiMemory<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let mem = static_init_half!(
            static_buffer.1,
            SpiMemory<'static>,
            SpiMemory::new(static_buffer.0)
        );
        static_buffer.0.set_client(mem);

        mem
    }
}
