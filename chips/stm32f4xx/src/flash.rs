use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;

/// Flash Memory interface
#[repr(C)]
struct FlashRegisters {
    /// SRAM/NOR-Flash chip-select control register
    acr: ReadWrite<u32, ACR::Register>,
}

register_bitfields![u32,
    ACR [
        /// Data cache reset
        DCRST OFFSET(12) NUMBITS(1) [],
        /// Instruction cache reset
        ICRST OFFSET(11) NUMBITS(1) [],
        /// Data cache enable
        DCEN OFFSET(10) NUMBITS(1) [],
        /// Instruction cache enable
        ICEN OFFSET(9) NUMBITS(1) [],
        /// Prefetch enable
        PRFTEN OFFSET(8) NUMBITS(1) [],
        /// Latency
        LATENCY OFFSET(0) NUMBITS(4) []
    ]
];

pub struct Flash {
    registers: StaticRef<FlashRegisters>,
}

impl Flash {
    const fn new(base_addr: StaticRef<FlashRegisters>) -> Flash {
        Flash {
            registers: base_addr,
        }
    }

    pub fn enable_instruction_cache(&self) {
        self.registers.acr.modify(ACR::ICEN::SET);
    }

    pub fn enable_data_cache(&self) {
        self.registers.acr.modify(ACR::DCEN::SET);
    }

    pub fn set_latency(&self, latency: u32) {
        self.registers.acr.modify(ACR::LATENCY.val(latency));
    }

    pub fn enable_prefetch(&self) {
        self.registers.acr.modify(ACR::PRFTEN::SET);
    }
}

const FLASH_BASE: StaticRef<FlashRegisters> =
    unsafe { StaticRef::new(0x4002_3C00 as *const FlashRegisters) };

pub static mut FLASH: Flash = Flash::new(FLASH_BASE);
