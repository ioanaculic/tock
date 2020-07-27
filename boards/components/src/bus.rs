use capsules::bus::I2CMasterBus;
use capsules::bus::SpiMasterBus;
use capsules::virtual_i2c::{I2CDevice, MuxI2C};
use capsules::virtual_spi::VirtualSpiMasterDevice;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::spi;
use kernel::static_init_half;
use kernel::hil::i2c;

// Setup static space for the objects.
#[macro_export]
macro_rules! spi_bus_component_helper {
    ($S:ty, $select:expr, $spi_mux: expr) => {{
        use capsules::bus::SpiMasterBus;
        use capsules::virtual_spi::VirtualSpiMasterDevice;
        use core::mem::{size_of, MaybeUninit};
        let bus_spi: &'static VirtualSpiMasterDevice<'static, $S> =
            components::spi::SpiComponent::new($spi_mux, $select)
                .finalize(components::spi_component_helper!($S));
        static mut ADDRESS_BUFFER: [u8; size_of::<usize>()] = [0; size_of::<usize>()];
        static mut bus: MaybeUninit<SpiMasterBus<'static, VirtualSpiMasterDevice<'static, $S>>> =
            MaybeUninit::uninit();
        (&bus_spi, &mut bus, &mut ADDRESS_BUFFER)
    };};
}

#[macro_export]
macro_rules! i2c_master_bus_component_helper {
    ($I: ty, $S: ty, $i2c_mux: expr, $address: expr) => {{
        use capsules::bus::I2CMasterBus;
        use core::mem::{size_of, MaybeUninit};
        use capsules::virtual_i2c::I2CDevice;
        let bus_i2c: &'static I2CDevice<'static, $I, $S> =
            components::i2c::I2CComponent::new($i2c_mux, $address)
                .finalize(components::i2c_component_helper!($I, $S));
        static mut ADDRESS_BUFFER: [u8; 1] = [0; 1];
        static mut bus: MaybeUninit<I2CMasterBus<'static, I2CDevice<'static, $I, $S>>> = MaybeUninit::uninit();
        (&bus_i2c, &mut bus, &mut ADDRESS_BUFFER)
    };};
}

pub struct SpiMasterBusComponent<S: 'static + spi::SpiMaster> {
    _select: PhantomData<S>,
}

impl<S: 'static + spi::SpiMaster> SpiMasterBusComponent<S> {
    pub fn new() -> SpiMasterBusComponent<S> {
        SpiMasterBusComponent {
            _select: PhantomData,
        }
    }
}

impl<S: 'static + spi::SpiMaster> Component for SpiMasterBusComponent<S> {
    type StaticInput = (
        &'static VirtualSpiMasterDevice<'static, S>,
        &'static mut MaybeUninit<SpiMasterBus<'static, VirtualSpiMasterDevice<'static, S>>>,
        &'static mut [u8],
    );
    type Output = &'static SpiMasterBus<'static, VirtualSpiMasterDevice<'static, S>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let bus = static_init_half!(
            static_buffer.1,
            SpiMasterBus<'static, VirtualSpiMasterDevice<'static, S>>,
            SpiMasterBus::new(static_buffer.0, static_buffer.2)
        );
        static_buffer.0.set_client(bus);

        bus
    }
}

pub struct I2CMasterBusComponent<I: 'static + i2c::I2CMaster, S: 'static + i2c::SMBusMaster> {
    _i2c: PhantomData<I>,
    _smbus: PhantomData<S>
}

impl<I: 'static + i2c::I2CMaster, S: 'static + i2c::SMBusMaster> I2CMasterBusComponent<I, S> {
    pub fn new() -> I2CMasterBusComponent<I, S> {
        I2CMasterBusComponent {
            _i2c: PhantomData,
            _smbus: PhantomData
        }
    }
}

impl<I: 'static + i2c::I2CMaster, S: 'static + i2c::SMBusMaster> Component for I2CMasterBusComponent<I, S> {
    type StaticInput = (
        &'static I2CDevice<'static, I, S>,
        &'static mut MaybeUninit<I2CMasterBus<'static, I2CDevice<'static, I, S>>>,
        &'static mut [u8],
    );
    type Output = &'static I2CMasterBus<'static, I2CDevice<'static, I, S>>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        

        let bus = static_init_half!(
            static_buffer.1,
            I2CMasterBus<'static, I2CDevice<'static, I, S>>,
            I2CMasterBus::new(static_buffer.0, static_buffer.2)
        );
        static_buffer.0.set_client(bus);

        bus
    }
}
