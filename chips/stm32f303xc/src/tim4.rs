use crate::gpio;
use crate::nvic;
use crate::pwm;
use crate::rcc;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::debug;
use kernel::hil;
use kernel::ClockInterface;
use kernel::ReturnCode;

/// General purpose timers
#[repr(C)]
struct Tim4Registers {
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
    /// capture/compare mode register 2 (output mode)
    ccmr2_output: ReadWrite<u32, CCMR2_Output::Register>,
    /// capture/compare enable register
    ccer: ReadWrite<u32, CCER::Register>,
    /// counter
    cnt: ReadWrite<u32, CNT::Register>,
    /// prescaler
    psc: ReadWrite<u32, PSC::Register>,
    /// auto-reload register
    arr: ReadWrite<u32, ARR::Register>,
    _reserved0: [u8; 4],
    /// capture/compare register 1
    ccr1: ReadWrite<u32, CCR1::Register>,
    /// capture/compare register 2
    ccr2: ReadWrite<u32, CCR2::Register>,
    /// capture/compare register 3
    ccr3: ReadWrite<u32, CCR3::Register>,
    /// capture/compare register 4
    ccr4: ReadWrite<u32, CCR4::Register>,
    _reserved1: [u8; 4],
    /// DMA control register
    dcr: ReadWrite<u32, DCR::Register>,
    /// DMA address for full transfer
    dmar: ReadWrite<u32, DMAR::Register>,
    /// TIM5 option register
    or_: ReadWrite<u32>,
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
        /// TI1 selection
        TI1S OFFSET(7) NUMBITS(1) [],
        /// Master mode selection
        MMS OFFSET(4) NUMBITS(3) [],
        /// Capture/compare DMA selection
        CCDS OFFSET(3) NUMBITS(1) []
    ],
    SMCR [
        /// Slave mode selection - bit 3
        SMS3 OFFSET(16) NUMBITS(1) [],
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
        /// Trigger interrupt enable
        TIE OFFSET(6) NUMBITS(1) [],
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
        /// Capture/Compare 4 overcapture flag
        CC4OF OFFSET(12) NUMBITS(1) [],
        /// Capture/Compare 3 overcapture flag
        CC3OF OFFSET(11) NUMBITS(1) [],
        /// Capture/compare 2 overcapture flag
        CC2OF OFFSET(10) NUMBITS(1) [],
        /// Capture/Compare 1 overcapture flag
        CC1OF OFFSET(9) NUMBITS(1) [],
        /// Trigger interrupt flag
        TIF OFFSET(6) NUMBITS(1) [],
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
        /// Trigger generation
        TG OFFSET(6) NUMBITS(1) [],
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
        /// OC2M bit 3
        OC2M3 OFFSET(24) NUMBITS(1) [],
        /// OC1M bit 3
        OC1M3 OFFSET(16) NUMBITS(1) [],
        /// OC2CE
        OC2CE OFFSET(15) NUMBITS(1) [],
        /// OC2M
        OC2M OFFSET(12) NUMBITS(3) [],
        /// OC2PE
        OC2PE OFFSET(11) NUMBITS(1) [],
        /// OC2FE
        OC2FE OFFSET(10) NUMBITS(1) [],
        /// CC2S
        CC2S OFFSET(8) NUMBITS(2) [],
        /// OC1CE
        OC1CE OFFSET(7) NUMBITS(1) [],
        /// OC1M
        OC1M OFFSET(4) NUMBITS(3) [],
        /// OC1PE
        OC1PE OFFSET(3) NUMBITS(1) [],
        /// OC1FE
        OC1FE OFFSET(2) NUMBITS(1) [],
        /// CC1S
        CC1S OFFSET(0) NUMBITS(2) []
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
        ICPCS OFFSET(2) NUMBITS(2) [],
        /// Capture/Compare 1 selection
        CC1S OFFSET(0) NUMBITS(2) []
    ],
    CCMR2_Output [
        /// OC4M bit 3
        OC4M3 OFFSET(24) NUMBITS(1) [],
        /// OC3M bit 3
        OC3M3 OFFSET(16) NUMBITS(1) [],
        /// OC4CE
        OC4CE OFFSET(15) NUMBITS(1) [],
        /// OC4M
        OC4M OFFSET(12) NUMBITS(3) [],
        /// OC4PE
        OC4PE OFFSET(11) NUMBITS(1) [],
        /// OC4FE
        OC4FE OFFSET(10) NUMBITS(1) [],
        /// CC4S
        CC4S OFFSET(8) NUMBITS(2) [],
        /// OC3CE
        OC3CE OFFSET(7) NUMBITS(1) [],
        /// OC3M
        OC3M OFFSET(4) NUMBITS(3) [],
        /// OC3PE
        OC3PE OFFSET(3) NUMBITS(1) [],
        /// OC3FE
        OC3FE OFFSET(2) NUMBITS(1) [],
        /// CC3S
        CC3S OFFSET(0) NUMBITS(2) []
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
        CC3S OFFSET(0) NUMBITS(2) []
    ],
    CCER [
        /// Capture/Compare 4 output Polarity
        CC4NP OFFSET(15) NUMBITS(1) [],
        /// Capture/Compare 3 output Polarity
        CC4P OFFSET(13) NUMBITS(1) [],
        /// Capture/Compare 4 output enable
        CC4E OFFSET(12) NUMBITS(1) [],
        /// Capture/Compare 3 output Polarity
        CC3NP OFFSET(11) NUMBITS(1) [],
        /// Capture/Compare 3 output Polarity
        CC3P OFFSET(9) NUMBITS(1) [],
        /// Capture/Compare 3 output enable
        CC3E OFFSET(8) NUMBITS(1) [],
        /// Capture/Compare 2 output Polarity
        CC2NP OFFSET(7) NUMBITS(1) [],
        /// Capture/Compare 2 output Polarity
        CC2P OFFSET(5) NUMBITS(1) [],
        /// Capture/Compare 2 output enable
        CC2E OFFSET(4) NUMBITS(1) [],
        /// Capture/Compare 1 output Polarity
        CC1NP OFFSET(3) NUMBITS(1) [],
        /// Capture/Compare 1 output Polarity
        CC1P OFFSET(1) NUMBITS(1) [],
        /// Capture/Compare 1 output enable
        CC1E OFFSET(0) NUMBITS(1) []
    ],
    CNT [
        /// High counter value
        CNT_H OFFSET(16) NUMBITS(16) [],
        /// Low counter value
        CNT_L OFFSET(0) NUMBITS(16) []
    ],
    PSC [
        /// Prescaler
        PSC OFFSET(0) NUMBITS(16) []
    ],
    ARR [
        ARR OFFSET(0) NUMBITS(16) []
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
        /// Capture/Compare 3 value
        CCR3 OFFSET(0) NUMBITS(16) []
    ],
    CCR4 [
        /// Capture/Compare 4 value
        CCR4 OFFSET(0) NUMBITS(16) []
    ],
    DCR [
        /// DMA burst length
        DBL OFFSET(8) NUMBITS(5) [],
        /// DMA base address
        DBA OFFSET(0) NUMBITS(5) []
    ],
    DMAR [
        /// DMA register for burst accesses
        DMAB OFFSET(0) NUMBITS(16) []
    ]
];

const TIM4_BASE: StaticRef<Tim4Registers> =
    unsafe { StaticRef::new(0x40000800 as *const Tim4Registers) };

pub struct Tim4 {
    registers: StaticRef<Tim4Registers>,
    clock: Tim4Clock,
}

pub static mut TIM4: Tim4 = Tim4::new();

impl Tim4 {
    const fn new() -> Self {
        Self {
            registers: TIM4_BASE,
            clock: Tim4Clock(rcc::PeripheralClock::APB1(rcc::PCLK1::TIM4)),
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

    pub fn start_pwm(&self, channel: u8, frequency: usize, duty_cycle: usize) -> ReturnCode {
        let mut ret = ReturnCode::SUCCESS;
        self.enable_clock();
        let counter_top = 8000000 / frequency;
        let dc_out = counter_top - (duty_cycle / frequency);
        self.registers.cr1.modify(CR1::DIR.val(0));
        self.registers.cr1.modify(CR1::CKD.val(0));
        self.registers.cr1.modify(CR1::ARPE::CLEAR);
        self.registers.cr1.modify(CR1::CMS.val(0));
        self.registers.psc.modify(PSC::PSC.val(0));
        self.registers
            .arr
            .modify(ARR::ARR.val((counter_top as u16 - 1).into()));
        if channel == 1 {
            self.registers.ccmr1_output.modify(CCMR1_Output::OC1PE::SET);
            self.registers.ccmr1_output.modify(CCMR1_Output::OC1CE::SET);
            self.registers
                .ccmr1_output
                .modify(CCMR1_Output::OC1M.val(0b110));
            self.registers.egr.write(EGR::UG::SET);
            self.registers
                .ccr1
                .modify(CCR1::CCR1.val((dc_out as u16).into()));
            self.registers.ccer.modify(CCER::CC1E::SET);
        } else if channel == 2 {
            self.registers.ccmr1_output.modify(CCMR1_Output::OC2PE::SET);
            self.registers.ccmr1_output.modify(CCMR1_Output::OC2CE::SET);
            self.registers
                .ccmr1_output
                .modify(CCMR1_Output::OC2M.val(0b110));
            self.registers.egr.write(EGR::UG::SET);
            self.registers
                .ccr2
                .modify(CCR2::CCR2.val((dc_out as u16).into()));
            self.registers.ccer.modify(CCER::CC2E::SET);
        } else if channel == 3 {
            self.registers.ccmr2_output.modify(CCMR2_Output::OC3PE::SET);
            self.registers.ccmr2_output.modify(CCMR2_Output::OC3CE::SET);
            self.registers
                .ccmr2_output
                .modify(CCMR2_Output::OC3M.val(0b110));
            self.registers.egr.write(EGR::UG::SET);
            self.registers
                .ccr3
                .modify(CCR3::CCR3.val((dc_out as u16).into()));
            self.registers.ccer.modify(CCER::CC3E::SET);
        } else if channel == 4 {
            self.registers.ccmr2_output.modify(CCMR2_Output::OC4PE::SET);
            self.registers.ccmr2_output.modify(CCMR2_Output::OC4CE::SET);
            self.registers
                .ccmr2_output
                .modify(CCMR2_Output::OC4M.val(0b110));
            self.registers.egr.write(EGR::UG::SET);
            self.registers
                .ccr4
                .modify(CCR4::CCR4.val((dc_out as u16).into()));
            self.registers.ccer.modify(CCER::CC4E::SET);
        } else {
            ret = ReturnCode::ENOSUPPORT
        }

        self.registers.cr1.modify(CR1::CEN::SET);

        ret
    }

    pub fn stop_pwm(&self, _channel: u8) -> ReturnCode {
        ReturnCode::SUCCESS
    }
}

struct Tim4Clock(rcc::PeripheralClock);

impl ClockInterface for Tim4Clock {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}

impl hil::pwm::Pwm for Tim4 {
    type Pin = pwm::TimPwmPin<'static>;

    fn start(&self, pin: &Self::Pin, frequency: usize, duty_cycle: usize) -> ReturnCode {
        self.start_pwm(pin.channel, frequency, duty_cycle)
    }

    fn stop(&self, pin: &Self::Pin) -> ReturnCode {
        self.stop_pwm(pin.channel)
    }

    fn get_maximum_frequency_hz(&self) -> usize {
        8000000
    }

    fn get_maximum_duty_cycle(&self) -> usize {
        8000000
    }
}
