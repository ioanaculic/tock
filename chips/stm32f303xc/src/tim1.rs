use cortexm4;
use cortexm4::support::atomic;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ClockInterface;

use crate::nvic;
use crate::rcc;

#[repr(C)]
struct Tim1Registers {
    /// control register 1
    cr1: ReadWrite<u32, CR1::Register>,
    /// control register 2
    cr2: ReadWrite<u32, CR2::Register>,
    /// slave mode control register
    smcr: ReadWrite<u32, SMCR::Register>,
    /// DMA/Interrupt enable register
    dier: ReadWrite<u32, DIER::Register>,
    /// status register
    sr: ReadWrite<u32, SR::Register>,
    /// event generation register
    egr: WriteOnly<u32, EGR::Register>,
    /// capture/compare mode register 1 (output mode)
    ccmr1_output: ReadWrite<u32, CCMR1_Output::Register>,
    /// capture/compare mode register 1 (input mode)
    ccmr1_input: ReadWrite<u32, CCMR1_Input::Register>,
    /// capture/compare mode register 2 (output mode)
    ccmr2_output: ReadWrite<u32, CCMR2_Output::Register>,
    /// capture/compare mode register 2 (input mode)
    ccmr2_input: ReadWrite<u32, CCMR2_Input::Register>,
    /// capture/compare enable register
    ccer: ReadWrite<u32, CCER::Register>,
    /// counter
    cnt: ReadWrite<u32, CNT::Register>,
    /// prescaler
    psc: ReadWrite<u32>,
    /// auto-reload register
    arr: ReadWrite<u32, ARR::Register>,
    /// repetition counter register
    rcr: ReadWrite<u32, RCR::Register>,
    /// capture/compare register 1
    ccr1: ReadWrite<u32, CCR1::Register>,
    /// capture/compare register 2
    ccr2: ReadWrite<u32, CCR2::Register>,
    /// capture/compare register 3
    ccr3: ReadWrite<u32, CCR3::Register>,
    /// capture/compare register 4
    ccr4: ReadWrite<u32, CCR4::Register>,
    /// break and dead-time register
    bdtr: ReadWrite<u32, BDTR::Register>,
    /// DMA control register
    dcr: ReadWrite<u32, DCR::Register>,
    /// DMA address for full transfer
    dmar: ReadWrite<u32, DMAR::Register>,
    /// TIM1 option register
    or: ReadWrite<u32, OR: Register>,
    /// capture/compare mode register 3 (output mode)
    ccmr3_output: ReadWrite<u32, CCMR3_Output::Register>,
    /// capture/compare register 5
    ccr5: ReadWrite<u32, CCR55::Register>,
    /// capture/compare register 6
    ccr6: ReadWrite<u32, CCR6::Register>,
}

register_bitfields![u32,
    CR1 [
        /// UIF status bit remapping
        UIFREMAP OFFSET(11) NUMBITS(1) [],
        /// Clock division
        CKD OFFSET(8) NUMBITS(2) [],
        /// Auto-reload preload enable
        ARPE OFFSET(7) NUMBITS(1) [],
        /// Center-aligned mode selection
        CMS OFFSET(5) NUMBITS(2) [],
        /// Direction
        DIR OFFSET(4) NUMBITS(1) [],
        /// One-pulse mode
        OPM OFFSET(3) NUMBITS(1) [],
        /// Update request source
        URS OFFSET(2) NUMBITS(1) [],
        /// Update disable
        UDIS OFFSET(1) NUMBITS(1) [],
        /// Counter enable
        CEN OFFSET(0) NUMBITS(1) []
    ],
    CR2 [
        /// Master mode selection 2
        MMS2 OFFSET(20) NUMBITS(3) [],
        /// Output Idle state 6 (OC6 output)
        OIS6 OFFSET(18) NUMBITS(1) [],
        /// Output Idle state 5 (OC5 output)
        OIS5 OFFSET(16) NUMBITS(1) [],
        /// Output Idle state 4 (OC4 output)
        OIS4 OFFSET(14) NUMBITS(1) [],
        /// Output Idle state 3 (OC3N output)
        OIS3N OFFSET(13) NUMBITS(1) [],
        /// Output Idle state 3 (OC3 output)
        OIS3 OFFSET(12) NUMBITS(1) [],
        /// Output Idle state 2 (OC2N output)
        OIS2N OFFSET(11) NUMBITS(1) [],
        /// Output Idle state 2 (OC2 output)
        OIS2 OFFSET(10) NUMBITS(1) [],
        /// Output Idle state 1 (OC1N output)
        OIS1N OFFSET(9) NUMBITS(1) [],
        /// Output Idle state 1 (OC1 output)
        OIS1 OFFSET(8) NUMBITS(1) [],
        /// TI1 selection
        TI1S OFFSET(7) NUMBITS(1) [],
        /// Master mode selection
        MMS OFFSET(4) NUMBITS(3) [],
        /// Capture/compare DMA selection
        CCDS OFFSET(3) NUMBITS(1) [],
        /// Capture/compare control update selection
        CCUS OFFSET(2) NUMBITS(1) [],
        /// Capture/compare preloaded control
        CCPC OFFSET(1) NUMBITS(1) [],
    ],
    SMCR [
        /// Slave mode selection - bit 3
        SMS OFFSET(16) NUMBITS(1) [],
        /// External trigger polarity
        ETP OFFSET(15) NUMBITS(1) [],
        /// External clock enable
        ECE OFFSET(14) NUMBITS(1) [],
        /// External trigger prescaler
        ETPS OFFSET(12) NUMBITS(2) [],
        /// External trigger filter
        ETF OFFSET(8) NUMBITS(4) [],
        /// Master/Slave mode
        MSM OFFSET(7) NUMBITS(1) [],
        /// Trigger selection
        TS OFFSET(4) NUMBITS(3) [],
        /// OCREF clear selection
        OCCS OFFSET(3) NUMBITS(1) [],
        /// Slave mode selection
        SMS OFFSET(0) NUMBITS(3) []
    ],
    DIER [
        /// Trigger DMA request enable
        TDE OFFSET(14) NUMBITS(1) [],
        /// COM DMA request enable
        COMDE OFFSET(13) NUMBITS(1) [],
        /// Capture/Compare 4 DMA request enable
        CC4DE OFFSET(12) NUMBITS(1) [],
        /// Capture/Compare 3 DMA request enable
        CC3DE OFFSET(11) NUMBITS(1) [],
        /// Capture/Compare 2 DMA request enable
        CC2DE OFFSET(10) NUMBITS(1) [],
        /// Capture/Compare 1 DMA request enable
        CC1DE OFFSET(9) NUMBITS(1) [],
        /// Update DMA request enable
        UDE OFFSET(8) NUMBITS(1) [],
        /// Break interrupt enable
        BIE OFFSET(7) NUMBITS(1) [],
        /// Trigger interrupt enable
        TIE OFFSET(6) NUMBITS(1) [],
        /// COM interrupt enable
        COMIE OFFSET(5) NUMBITS(1) [],
        /// Capture/Compare 4 interrupt enable
        CC4IE OFFSET(4) NUMBITS(1) [],
        /// Capture/Compare 3 interrupt enable
        CC3IE OFFSET(3) NUMBITS(1) [],
        /// Capture/Compare 2 interrupt enable
        CC2IE OFFSET(2) NUMBITS(1) [],
        /// Capture/Compare 1 interrupt enable
        CC1IE OFFSET(1) NUMBITS(1) [],
        /// Update interrupt enable
        UIE OFFSET(0) NUMBITS(1) []
    ],
    SR [
        /// Compare 6 interrupt flag
        CC6IF OFFSET(17) NUMBITS(1) [],
        /// Compare 5 interrupt flag
        CC5IF OFFSET(16) NUMBITS(1) [],
        /// Capture/Compare 4 overcapture flag
        CC4OF OFFSET(12) NUMBITS(1) [],
        /// Capture/Compare 3 overcapture flag
        CC3OF OFFSET(11) NUMBITS(1) [],
        /// Capture/compare 2 overcapture flag
        CC2OF OFFSET(10) NUMBITS(1) [],
        /// Capture/Compare 1 overcapture flag
        CC1OF OFFSET(9) NUMBITS(1) [],
        /// Break 2 interrupt flag
        B2IF OFFSET(8) NUMBITS(1) [],
        /// Break interrupt flag
        BIF OFFSET(7) NUMBITS(1) [],
        /// Trigger interrupt flag
        TIF OFFSET(6) NUMBITS(1) [],
        /// COM interrupt flag
        COMIF OFFSET(5) NUMBITS(1) [],
        /// Capture/Compare 4 interrupt flag
        CC4IF OFFSET(4) NUMBITS(1) [],
        /// Capture/Compare 3 interrupt flag
        CC3IF OFFSET(3) NUMBITS(1) [],
        /// Capture/Compare 2 interrupt flag
        CC2IF OFFSET(2) NUMBITS(1) [],
        /// Capture/compare 1 interrupt flag
        CC1IF OFFSET(1) NUMBITS(1) [],
        /// Update interrupt flag
        UIF OFFSET(0) NUMBITS(1) []
    ],
    EGR [
        /// Break 2 generation
        B2G OFFSET(8) NUMBITS(1) [],
        /// Break generation
        BG OFFSET(7) NUMBITS(1) [],
        /// Trigger generation
        TG OFFSET(6) NUMBITS(1) [],
        /// Trigger generation
        COMG OFFSET(5) NUMBITS(1) [],
        /// Capture/compare 4 generation
        CC4G OFFSET(4) NUMBITS(1) [],
        /// Capture/compare 3 generation
        CC3G OFFSET(3) NUMBITS(1) [],
        /// Capture/compare 2 generation
        CC2G OFFSET(2) NUMBITS(1) [],
        /// Capture/compare 1 generation
        CC1G OFFSET(1) NUMBITS(1) [],
        /// Update generation
        UG OFFSET(0) NUMBITS(1) []
    ],
    CCMR1_Output [
        /// Output Compare 2 mode - bit 3
        OC2M OFFSET(24) NUMBITS(1) [],
        /// Output Compare 1 mode - bit 3
        OC1M OFFSET(16) NUMBITS(1) [],
        /// Output Compare 2 clear enable
        OC2CE OFFSET(15) NUMBITS(1) [],
        /// Output Compare 2 mode
        OC2M OFFSET(12) NUMBITS(3) [],
        /// Output Compare 2 preload enable
        OC2PE OFFSET(11) NUMBITS(1) [],
        /// Output Compare 2 fast enable
        OC2FE OFFSET(10) NUMBITS(1) [],
        /// Capture/Compare 2 selection
        CC2S OFFSET(8) NUMBITS(1) [],
        /// Output Compare 1 clear enable
        OC1CE OFFSET(7) NUMBITS(1) [],
        /// Output Compare 1 mode
        OC1M OFFSET(4) NUMBITS(3) [],
        /// Output Compare 1 preload enable
        OC1PE OFFSET(3) NUMBITS(1) [],
        /// Output Compare 1 fast enable
        OC1FE OFFSET(2) NUMBITS(1) [],
        /// Capture/Compare 1 selection
        CC1S OFFSET(0) NUMBITS(2) [],
    ],
    CCMR1_Input [
        /// Input capture 2 filter
        IC2F OFFSET(12) NUMBITS(4) [],
        /// Input capture 2 prescaler
        IC2PCS OFFSET(10) NUMBITS(2) [],
        /// Capture/Compare 2 selection
        CC2S OFFSET(8) NUMBITS(2) [],
        /// Input capture 1 filter
        IC1F OFFSET(4) NUMBITS(4) [],
        /// Input capture 1 prescaler
        IC1PSC OFFSET(2) NUMBITS(2) [],
        /// Capture/Compare 1 selection
        CC1S OFFSET(0) NUMBITS(2) []
    ],
    CCMR2_Output [
        /// Output Compare 4 mode - bit 3
        OC4M OFFSET(24) NUMBITS(1) [],
        /// Output Compare 3 mode - bit 3
        OC3M OFFSET(16) NUMBITS(1) [],
        /// Output compare 4 clear enable
        OC4CE OFFSET(15) NUMBITS(1) [],
        /// Output compare 4 mode
        OC4M OFFSET(12) NUMBITS(3) [],
        /// Output compare 4 preload enable
        OC4PE OFFSET(11) NUMBITS(1) [],
        /// Output compare 4 fast enable
        OC4FE OFFSET(10) NUMBITS(1) [],
        /// Capture/Compare 4 selection
        CC4S OFFSET(8) NUMBITS(2) [],
        /// Output compare 3 clear enable
        OC3CE OFFSET(7) NUMBITS(1) [],
        /// Output compare 3 mode
        OC3M OFFSET(4) NUMBITS(3) [],
        /// Output compare 3 preload enable
        OC3PE OFFSET(3) NUMBITS(1) [],
        /// Output compare 3 fast enable
        OC3FE OFFSET(2) NUMBITS(1) [],
        /// Capture/Compare 3 selection
        CC3S OFFSET(0) NUMBITS(2) [],
    ],
    CCMR2_Input [
        /// Input capture 4 filter
        IC4F OFFSET(12) NUMBITS(4) [],
        /// Input capture 4 prescaler
        IC4PSC OFFSET(10) NUMBITS(2) [],
        /// Capture/Compare 4 selection
        CC4S OFFSET(8) NUMBITS(2) [],
        /// Input capture 3 filter
        IC3F OFFSET(4) NUMBITS(4) [],
        /// Input capture 3 prescaler
        IC3PSC OFFSET(2) NUMBITS(2) [],
        /// Capture/compare 3 selection
        CC3S OFFSET(0) NUMBITS(0) [],
    ],
    CCER [
        /// Capture/Compare 6 output polarity
        CC6P OFFSET(21) NUMBITS(1) [],
        /// Capture/Compare 6 output enable
        CC6E OFFSET(20) NUMBITS(1) [],
        /// Capture/Compare 5 output polarity
        CC5P OFFSET(17) NUMBITS(1) [],
        /// Capture/Compare 5 output enable
        CC5E OFFSET(16) NUMBITS(1) [],
        /// Capture/Compare 4 complementary output polarity
        CC4NP OFFSET(15) NUMBITS(1) [],
        /// Capture/Compare 4 output polarity
        CC4P OFFSET(13) NUMBITS(1) [],
        /// Capture/Compare 4 output enable
        CC4E OFFSET(12) NUMBITS(1) [],
        /// Capture/Compare 3 complementary output polarity
        CC3NP OFFSET(11) NUMBITS(1) [],
        /// Capture/Compare 3 complementary output enable
        CC3NE OFFSET(10) NUMBITS(1) [],
        /// Capture/Compare 3 output polarity
        CC3P OFFSET(9) NUMBITS(1) [],
        /// Capture/Compare 3 output enable
        CC3E OFFSET(8) NUMBITS(1) [],
        /// Capture/Compare 2 complementary output polarity
        CC2NP OFFSET(7) NUMBITS(1) [],
        /// Capture/Compare 2 complementary output enable
        CC2NE OFFSET(6) NUMBITS(1) [],
        /// Capture/Compare 2 output Polarity
        CC2P OFFSET(5) NUMBITS(1) [],
        /// Capture/Compare 2 output enable
        CC2E OFFSET(4) NUMBITS(1) [],
        /// Capture/Compare 1 complementary output polarity
        CC1NP OFFSET(3) NUMBITS(1) [],
        /// Capture/Compare 1 complementary output enable
        CC1NE OFFSET(2) NUMBITS(1) [],
        /// Capture/Compare 1 output Polarity
        CC1P OFFSET(1) NUMBITS(1) [],
        /// Capture/Compare 1 output enable
        CC1E OFFSET(0) NUMBITS(1) []
    ],
    CNT [
        /// UIF copy
        UIFCPY OFFSET(31) NUMBITS(1) [],
        /// Counter value
        CNT OFFSET(0) NUMBITS(16) []
    ],
    PSC [
        /// Prescaler value
        PSC OFFSET(0) NUMBITS(16) [],
    ],
    ARR [
        /// Prescaler value
        ARR OFFSET(0) NUMBITS(16) [],
    ],
    RCR [
        /// Repetition counter
        REP OFFSET(0) NUMBITS(16) [],
    ],
    CCR1 [
        /// Capture/Compare 1 value
        CCR1 OFFSET(0) NUMBITS(16) []
    ],
    CCR2 [
        /// Capture/Compare 2 value
        CCR2 OFFSET(0) NUMBITS(16) []
    ],
    CCR3 [
        /// Capture/Compare value
        CCR3 OFFSET(0) NUMBITS(16) []
    ],
    CCR4 [
        /// Capture/Compare value
        CCR4 OFFSET(0) NUMBITS(16) []
    ],
    BDTR [
        /// Break 2 polarity
        BK2P OFFSET(25) NUMBITS(1) [],
        /// Break 2 enable
        BK2E OFFSET(24) NUMBITS(1) [],
        /// Break 2 filter
        BK2F OFFSET(20) NUMBITS(4) [],
        /// Break filter
        BKF OFFSET(16) NUMBITS(4) [],
        /// Main output enable
        MOE OFFSET(15) NUMBITS(1) [],
        /// Automatic output enable
        AOE OFFSET(14) NUMBITS(1) [],
        /// Break polarity
        BKP OFFSET(13) NUMBITS(1) [],
        /// Break enable
        BKE OFFSET(12) NUMBITS(1) [],
        /// Off-state selection for Run mode
        OSSR OFFSET(11) NUMBITS(1) [],
        /// Off-state selection for Idle mode
        OSSI OFFSET(10) NUMBITS(1) [],
        /// Lock configuration
        LOCK OFFSET(8) NUMBITS(2) [],
        /// Dead-time generator setup
        DTG OFFSET(0) NUMBITS(8) [],
    ],
    DCR [
        /// DMA burst length
        DBL OFFSET(8) NUMBITS(5) [],
        /// DMA base address
        DBA OFFSET(0) NUMBITS(5) []
    ],
    DMAR [
        /// DMA register for burst accesses
        DMAB OFFSET(0) NUMBITS(32) [],
    ],
    OR [
        /// TIM1_ETR_ADC4 remapping capability
        TIM1_ETR_ADC4_RMP OFFSET(2) NUMBITS(2) [],
        /// TIM1_ETR_ADC1 remapping capability
        TIM1_ETR_ADC1_RMP OFFSET(0) NUMBITS(2) [],
    ],
    CCMR3_Output [
        /// Output Compare 6 mode - bit 3
        OC6M OFFSET(24) NUMBITS(1) [],
        /// Output Compare 5 mode - bit 3
        OC5M OFFSET(16) NUMBITS(1) [],
        /// Output compare 6 clear enable
        OC6CE OFFSET(15) NUMBITS(1) [],
        /// Output compare 6 mode
        OC6M OFFSET(12) NUMBITS(3) [],
        /// Output compare 6 preload enable
        OC6PE OFFSET(11) NUMBITS(1) [],
        /// Output compare 6 fast enable
        OC6FE OFFSET(10) NUMBITS(1) [],
        /// Output compare 5 clear enable
        OC5CE OFFSET(7) NUMBITS(1) [],
        /// Output compare 5 mode
        OC5M OFFSET(4) NUMBITS(3) [],
        /// Output compare 5 preload enable
        OC5PE OFFSET(3) NUMBITS(1) [],
        /// Output compare 5 fast enable
        OC5FE OFFSET(2) NUMBITS(1) [],
    ],
    CCR5 [
        /// Group Channel 5 and Channel 3
        GC5C3 OFFSET(31) NUMBITS(1) [],
        /// Group Channel 5 and Channel 2
        GC5C2 OFFSET(30) NUMBITS(1) [],
        /// Group Channel 5 and Channel 1
        GC5C1 OFFSET(29) NUMBITS(1) [],
        /// Capture/Compare 5 value
        CCR5 OFFSET(0) NUMBITS(16) [],
    ],
    CCR6 [
        /// Capture/Compare 6 value
        CCR6 OFFSET(0) NUMBITS(16) [],
    ]
];

const TIM1_BASE: StaticRef<Tim1Registers> =
    unsafe { StaticRef::new(0x4001_2C00 as *const Tim1Registers) };

pub struct Tim2<'a> {
    registers: StaticRef<Tim1Registers>,
    clock: Tim1Clock,
    // client: OptionalCell<&'a dyn hil::time::AlarmClient>,
    // irqn: u32,
}

pub static mut TIM1: Tim1<'static> = Tim1::new();

impl Tim1<'_> {
    const fn new() -> Self {
        Self {
            registers: TIM1_BASE,
            clock: Tim1Clock(rcc::PeripheralClock::APB2(rcc::PCLK2::TIM1)),
            // client: OptionalCell::empty(),
            // irqn: nvic::TIM2,
        }
    }

    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }

    pub fn handle_interrupt(&self) {}

    // starts the timer
    pub fn start(&self) {
        // TIM2 uses PCLK1. By default PCLK1 uses HSI running at 8Mhz.
        // Before calling set_alarm, we assume clock to TIM2 has been
        // enabled.

        self.registers.arr.set(0xFFFF_FFFF - 1);
        // Prescale 8Mhz to 16Khz, by dividing it by 500. We need set EGR.UG
        // in order for the prescale value to become active.
        self.registers.psc.set((499 - 1) as u32);
        self.registers.egr.write(EGR::UG::SET);
        self.registers.cr1.modify(CR1::CEN::SET);
    }
}
