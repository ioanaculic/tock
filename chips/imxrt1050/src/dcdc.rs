use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;

/// DCDC Converter module
#[repr(C)]
struct DcdcRegisters {
    reg0: ReadWrite<u32, REG0::Register>,
    reg1: ReadWrite<u32, REG1::Register>,
    reg2: ReadWrite<u32, REG2::Register>,
    reg3: ReadWrite<u32, REG3::Register>,
}

register_bitfields![u32,
	REG0[
        // DCDC Output OK
        STS_DC_OK OFFSET(31) NUMBITS(1) [],
        // 24M XTAL OK
        XTAL_24M_OK OFFSET(29) NUMBITS(1) [],
        // Reset Current Alert Signal
        CURRENT_ALERT_RESET OFFSET(27) NUMBITS(1) [],
        // Disable xtalok detection circuit
        XTALOK_DISABLE OFFSET(27) NUMBITS(1) [],
        // Power down output range comparator
        PWD_CMP_OFFSET OFFSET(26) NUMBITS(1) [],
        // Low Power High Hysteric Value
        LP_HIGH_HYS OFFSET(21) NUMBITS(1) [],
        // Low Power Overload Frequency Select
        LP_OVERLOAD_FREQ_SEL OFFSET(20) NUMBITS(1) [],
        // Low Power Overload Threshold
        LP_OVERLOAD_THRSH OFFSET(18) NUMBITS(2) [],
        // Power Down High Voltage Detection
        PWD_HIGH_VOLT_DET OFFSET(17) NUMBITS(1) [],
        // Low Power Overload Sense Enable
        EN_LP_OVERLOAD_SNS OFFSET(16) NUMBITS(1) [],
        // Power Down Battery Detection Comparator
        PWD_CMP_BATT_DET OFFSET(11) NUMBITS(1) [],
        // Overcurrent Trigger Adjust
        OVERCUR_TRIG_ADJ OFFSET(9) NUMBITS(2) [],
        // Power down overcurrent detection comparator
        PWD_OVERCUR_DET OFFSET(8) NUMBITS(1) [],
        // Current Sense (detector) Threshold
        CUR_SNS_THRSH OFFSET(5) NUMBITS(3) [],
        // Power down signal of the current detector.
        PWD_CUR_SNS_CMP OFFSET(4) NUMBITS(1) [],
        // Power down internal osc
        PWD_OSC_INT OFFSET(3) NUMBITS(1) [],
        // Select clock
        SEL_CLK OFFSET(2) NUMBITS(1) [],
        // Disable Auto Clock Switch
        DISABLE_AUTO_CLK_SWITCH OFFSET(1) NUMBITS(1) [],
        // Power Down Zero Cross Detection
        PWD_ZCD OFFSET(0) NUMBITS(1) []
    ],

    REG1 [
    	// Trim Bandgap Voltage
        VBG_TRIM OFFSET(24) NUMBITS(5) [],
        // Enable Hysteresis
        LOOPCTRL_EN_HYST OFFSET(23) NUMBITS(1) [],
        // Increase Threshold Detection
        LOOPCTRL_HST_THRESH OFFSET(21) NUMBITS(1) [],
        // Low Power Comparator Current Bias
        LP_CMP_ISRC_SEL OFFSET(12) NUMBITS(2) [],
        // Controls the load resistor of the internal regulator of DCDC
        REG_RLOAD_SW OFFSET(9) NUMBITS(1) [],
        // Selects the feedback point of the internal regulator
        REG_FBK_SEL OFFSET(7) NUMBITS(2) []
    ],

    REG2 [
    	// DCM Set Control
        DCM_SET_CTRL OFFSET(28) NUMBITS(1) [],
        // Disable Pulse Skip
        DISABLE_PULSE_SKIP OFFSET(27) NUMBITS(1) [],
        // Increase Threshold Detection
        LOOPCTRL_HYST_SIGN OFFSET(21) NUMBITS(1) [],
        // Increase the threshold detection for RC scale circuit.
        LOOPCTRL_RCSCALE_THRSH OFFSET(12) NUMBITS(1) [],
        // Enable RC Scale
        LOOPCTRL_EN_RCSCALE OFFSET(9) NUMBITS(2) [],
        // Two's complement feed forward step in duty cycle in the switching DC-DC converter
        LOOPCTRL_DC_FF OFFSET(6) NUMBITS(3) []
    ],

    REG3 [
    	// Disable Step
        DISABLE_STEP OFFSET(30) NUMBITS(1) [],
        // Set DCDC clock to half frequency for continuous mode
        DISABLE_PULSE_SKIP OFFSET(24) NUMBITS(1) [],
        // Low Power Target Value
        TARGET_LP OFFSET(8) NUMBITS(3) [],
        // Target value of VDD_SOC
        TRG OFFSET(0) NUMBITS(5) []
    ]
];

const DCDC_BASE: StaticRef<DcdcRegisters> =
    unsafe { StaticRef::new(0x40080000 as *const DcdcRegisters) };

pub struct Dcdc {
    registers: StaticRef<DcdcRegisters>,
}

pub static mut DCDC: Dcdc = Dcdc::new();

impl Dcdc {
    const fn new() -> Dcdc {
        Dcdc {
            registers: DCDC_BASE,
        }
    }

    pub fn modify_vdd_soc(&self, value: u32) {
    	let trg = (value - 800) / 25;
    	self.registers.reg3.modify(REG3::TRG.val(trg as u32));
    	while self.registers.reg0.read(REG0::STS_DC_OK) == 0 {}
    }
}
