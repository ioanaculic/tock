use core::mem::MaybeUninit;
use capsules::virtual_i2c::{MuxI2C, I2CDevice};
use capsules::mcp230xx::MCP230xx;
use kernel::component::Component;
use kernel::static_init_half;

#[macro_export]
macro_rules! mcp230xx_component_helper {
    () => {{
        use capsules::mcp230xx::MCP230xx;
        use capsules::virtual_i2c::I2CDevice;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<I2CDevice<'static>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<MCP230xx<'static>> = MaybeUninit::uninit();
        (
            &mut BUF1,
            &mut BUF2,
        )
    };};
}

pub struct MCP230XXComponent {
    mux_i2c: &'static MuxI2C<'static>,
}

impl MCP230XXComponent {
    pub fn new(mux_i2c: &'static MuxI2C<'static>) -> MCP230XXComponent {
        MCP230XXComponent {
            mux_i2c: mux_i2c
        }
    }
}

impl Component for MCP230XXComponent {
    type StaticInput = (
        &'static mut MaybeUninit<I2CDevice<'static>>,
        &'static mut MaybeUninit<MCP230xx<'static>>,
    );
    type Output = &'static MCP230xx<'static>;

    unsafe fn finalize(self, static_input: Self::StaticInput) -> Self::Output {
        let mcp230xx_i2c = static_init_half!(
            static_input.0,
            I2CDevice<'static>,
            I2CDevice::new(&self.mux_i2c, 0x20)
        );

        let mcp230xx = static_init_half!(
            static_input.1,
            MCP230xx<'static>,
            MCP230xx::new(
                mcp230xx_i2c, 
                None, 
                None, 
                &mut capsules::mcp230xx::BUFFER, 
                8, 
                1,
            )
        );
        mcp230xx_i2c.set_client(mcp230xx);

        mcp230xx
    }
}

