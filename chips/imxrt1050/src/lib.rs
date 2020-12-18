//! Peripheral implementations for the IMXRT1050 MCU.
//!
//! imxrt1050 chip: <https://www.nxp.com/design/development-boards/i-mx-evaluation-and-development-boards/i-mx-rt1050-evaluation-kit:MIMXRT1050-EVK>

#![crate_name = "imxrt1050"]
#![crate_type = "rlib"]
#![feature(llvm_asm, const_fn)]
#![no_std]

pub mod chip;
pub mod nvic;

// Peripherals
pub mod ccm;
pub mod gpio;
pub mod gpt1;
pub mod iomuxc;
pub mod iomuxc_snvs;
pub mod lpi2c;
pub mod lpuart;

use cortexm7::{generic_isr, hard_fault_handler, svc_handler, systick_handler};
// use kernel::common::registers::{register_bitfields, ReadWrite};
// use kernel::common::StaticRef;
use cortex_m_semihosting::hprintln;

// use cortexm::scb::{set_vector_table_offset};

#[cfg(not(any(target_arch = "arm", target_os = "none")))]
unsafe extern "C" fn unhandled_interrupt() {
    unimplemented!()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe extern "C" fn unhandled_interrupt() {
    let mut interrupt_number: u32;

    // IPSR[8:0] holds the currently active interrupt
    llvm_asm!(
    "mrs    r0, ipsr                    "
    : "={r0}"(interrupt_number)
    :
    : "r0"
    :
    );

    interrupt_number = interrupt_number & 0x1ff;

    panic!("Unhandled Interrupt. ISR {} is active.", interrupt_number);
}

extern "C" {
    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    fn _estack();

    // Defined by platform
    fn reset_handler();
}

#[cfg_attr(
    all(target_arch = "arm", target_os = "none"),
    link_section = ".vectors"
)]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static BASE_VECTORS: [unsafe extern "C" fn(); 16] = [
    _estack,
    reset_handler,
    unhandled_interrupt, // NMI
    hard_fault_handler,  // Hard Fault
    unhandled_interrupt, // MemManage
    unhandled_interrupt, // BusFault
    unhandled_interrupt, // UsageFault
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    svc_handler,         // SVC
    unhandled_interrupt, // DebugMon
    unhandled_interrupt,
    unhandled_interrupt, // PendSV
    systick_handler,     // SysTick
];

// imxrt 1050 has total of 160 interrupts
#[cfg_attr(all(target_arch = "arm", target_os = "none"), link_section = ".irqs")]
// used Ensures that the symbol is kept until the final binary
#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static IRQS: [unsafe extern "C" fn(); 160] = [
    generic_isr, // eDMA (0)
    generic_isr, // eDMA (1)
    generic_isr, // eDMA (2)
    generic_isr, // eDMA (3)
    generic_isr, // eDMA (4)
    generic_isr, // eDMA (5)
    generic_isr, // eDMA (6)
    generic_isr, // eDMA (7)
    generic_isr, // eDMA (8)
    generic_isr, // eDMA (9)
    generic_isr, // eDMA (10)
    generic_isr, // eDMA (11)
    generic_isr, // eDMA (12)
    generic_isr, // eDMA (13)
    generic_isr, // eDMA (14)
    generic_isr, // eDMA (15)
    generic_isr, // Error_interrupt (16)
    generic_isr, // CM7 (17)
    generic_isr, // CM7 (18)
    generic_isr, // CM7 (19)
    generic_isr, // LPUART1 (20)
    generic_isr, // LPUART2 (21)
    generic_isr, // LPUART3 (22)
    generic_isr, // LPUART4 (23)
    generic_isr, // LPUART5 (24)
    generic_isr, // LPUART6 (25)
    generic_isr, // LPUART7 (26)
    generic_isr, // LPUART8 (27)
    generic_isr, // LPI2C1 (28)
    generic_isr, // LPI2C2 (29)
    generic_isr, // LPI2C3 (30)
    generic_isr, // LPI2C4 (31)
    generic_isr, // LPSPI1 (32)
    generic_isr, // LPSPI2 (33)
    generic_isr, // LPSPI3 (34)
    generic_isr, // LPSPI4 (35)
    generic_isr, // FLEXCAN1 (36)
    generic_isr, // FLEXCAN2 (37)
    generic_isr, // CM7 (38)
    generic_isr, // KPP (39)
    generic_isr, // TSC_DIG (40)
    generic_isr, // GPR_IRQ (41)
    generic_isr, // LCDIF (42)
    generic_isr, // CSI (43)
    generic_isr, // PXP (44)
    generic_isr, // WDOG2 (45)
    generic_isr, // SNVS_HP_WRAPPER (46)
    generic_isr, // SNVS_HP_WRAPPER (47)
    generic_isr, // SNVS_HP_WRAPPER / SNVS_LP_WRAPPER (48)
    generic_isr, // CSU (49)
    generic_isr, // DCP (50)
    generic_isr, // DCP (51)
    generic_isr, // DCP (52)
    generic_isr, // TRNG (53)
    generic_isr, // Reserved (54)
    generic_isr, // BEE (55)
    generic_isr, // SAI1 (56)
    generic_isr, // SAI2 (57)
    generic_isr, // SAI3 (58)
    generic_isr, // SAI3 (59)
    generic_isr, // SPDIF (60)
    generic_isr, // PMU (61)
    generic_isr, // Reserved (62)
    generic_isr, // Temperature Monitor (63)
    generic_isr, // Temperature Monitor (64)
    generic_isr, // USB PHY (65)
    generic_isr, // USB PHY (66)
    generic_isr, // ADC1 (67)
    generic_isr, // ADC2 (68)
    generic_isr, // DCDC (69)
    generic_isr, // Reserved (70)
    generic_isr, // Reserved (71)
    generic_isr, // GPIO1 (72)
    generic_isr, // GPIO1 (73)
    generic_isr, // GPIO1 (74)
    generic_isr, // GPIO1 (75)
    generic_isr, // GPIO1 (76)
    generic_isr, // GPIO1 (77)
    generic_isr, // GPIO1 (78)
    generic_isr, // GPIO1 (79)
    generic_isr, // GPIO1_1 (80)
    generic_isr, // GPIO1_2 (81)
    generic_isr, // GPIO2_1 (82)
    generic_isr, // GPIO2_2 (83)
    generic_isr, // GPIO3_1 (84)
    generic_isr, // GPIO3_2 (85)
    generic_isr, // GPIO4_1 (86)
    generic_isr, // GPIO4_2 (87)
    generic_isr, // GPIO5_1 (88)
    generic_isr, // GPIO5_2 (89)
    generic_isr, // FLEXIO1 (90)
    generic_isr, // FLEXIO2 (91)
    generic_isr, // WDOG1 (92)
    generic_isr, // RTWDOG (93)
    generic_isr, // EWM (94)
    generic_isr, // CCM (95)
    generic_isr, // CCM (96)
    generic_isr, // GPC (97)
    generic_isr, // SRC (98)
    generic_isr, // Reserved (99)
    generic_isr, // GPT1 (100)
    generic_isr, // GPT2 (101)
    generic_isr, // FLEXPWM1 (102)
    generic_isr, // FLEXPWM1 (103)
    generic_isr, // FLEXPWM1 (104)
    generic_isr, // FLEXPWM1 (105)
    generic_isr, // FLEXPWM1 (106)
    generic_isr, // Reserved (107)
    generic_isr, // FLEXSPI (108)
    generic_isr, // SEMC (109)
    generic_isr, // USDHC1 (110)
    generic_isr, // USDHC2 (111)
    generic_isr, // USB (112)
    generic_isr, // USB (113)
    generic_isr, // ENET (114)
    generic_isr, // ENET (115)
    generic_isr, // XBAR1 (116)
    generic_isr, // XBAR1 (117)
    generic_isr, // ADC_ETC (118)
    generic_isr, // ADC_ETC (119)
    generic_isr, // ADC_ETC (120)
    generic_isr, // ADC_ETC (121)
    generic_isr, // PIT (122)
    generic_isr, // ACMP (123)
    generic_isr, // ACMP (124)
    generic_isr, // ACMP (125)
    generic_isr, // ACMP (126)
    generic_isr, // Reserved (127)
    generic_isr, // Reserved (128)
    generic_isr, // ENC1 (129)
    generic_isr, // ENC2 (130)
    generic_isr, // ENC3 (131)
    generic_isr, // ENC4 (132)
    generic_isr, // QTIMER1 (133)
    generic_isr, // QTIMER2 (134)
    generic_isr, // QTIMER3 (135)
    generic_isr, // QTIMER4 (136)
    generic_isr, // FLEXPWM2 (137)
    generic_isr, // FLEXPWM2 (138)
    generic_isr, // FLEXPWM2 (139)
    generic_isr, // FLEXPWM2 (140)
    generic_isr, // FLEXPWM2 (141)
    generic_isr, // FLEXPWM3 (142)
    generic_isr, // FLEXPWM3 (143)
    generic_isr, // FLEXPWM3 (144)
    generic_isr, // FLEXPWM3 (145)
    generic_isr, // FLEXPWM3 (146)
    generic_isr, // FLEXPWM4 (147)
    generic_isr, // FLEXPWM4 (148)
    generic_isr, // FLEXPWM4 (149)
    generic_isr, // FLEXPWM4 (150)
    generic_isr, // FLEXPWM4 (151)
    generic_isr, // Reserved (152)
    generic_isr, // Reserved (153)
    generic_isr, // Reserved (154)
    generic_isr, // Reserved (155)
    generic_isr, // Reserved (156)
    generic_isr, // Reserved (157)
    generic_isr, // Reserved (158)
    generic_isr, // Reserved (159)
];

extern "C" {
    static mut _szero: usize;
    static mut _ezero: usize;
    static mut _etext: usize;
    static mut _srelocate: usize;
    static mut _erelocate: usize;
}

// struct InterruptHandlers {
//     interrupt: [ReadWrite<u32, INTERRUPT::Register>; 42]
// }

// register_bitfields![u32,
//     INTERRUPT [
//         VAL OFFSET(0) NUMBITS(32) []
//     ]
// ];
// const INTERRUPT_BASE: StaticRef<InterruptHandlers> =
//     unsafe { StaticRef::new(0x60002040 as *const InterruptHandlers) };

// #[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe extern "C" fn my_generic_isr() {
    hprintln!("Cica crapa, cel putin asa pare").unwrap();
    panic!("Suntem in handler-ul meu!");
}

#[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
pub static NEW_IRQS: [unsafe extern "C" fn(); 176] = [
    _estack,
    reset_handler,
    unhandled_interrupt, // NMI
    hard_fault_handler,  // Hard Fault
    unhandled_interrupt, // MemManage
    unhandled_interrupt, // BusFault
    unhandled_interrupt, // UsageFault
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    unhandled_interrupt,
    svc_handler,         // SVC
    unhandled_interrupt, // DebugMon
    unhandled_interrupt,
    unhandled_interrupt, // PendSV
    systick_handler,     // SysTick
    my_generic_isr, // eDMA (0)
    my_generic_isr, // eDMA (1)
    my_generic_isr, // eDMA (2)
    my_generic_isr, // eDMA (3)
    my_generic_isr, // eDMA (4)
    my_generic_isr, // eDMA (5)
    my_generic_isr, // eDMA (6)
    my_generic_isr, // eDMA (7)
    my_generic_isr, // eDMA (8)
    my_generic_isr, // eDMA (9)
    my_generic_isr, // eDMA (10)
    my_generic_isr, // eDMA (11)
    my_generic_isr, // eDMA (12)
    my_generic_isr, // eDMA (13)
    my_generic_isr, // eDMA (14)
    my_generic_isr, // eDMA (15)
    my_generic_isr, // Error_interrupt (16)
    my_generic_isr, // CM7 (17)
    my_generic_isr, // CM7 (18)
    my_generic_isr, // CM7 (19)
    my_generic_isr, // LPUART1 (20)
    my_generic_isr, // LPUART2 (21)
    my_generic_isr, // LPUART3 (22)
    my_generic_isr, // LPUART4 (23)
    my_generic_isr, // LPUART5 (24)
    my_generic_isr, // LPUART6 (25)
    my_generic_isr, // LPUART7 (26)
    my_generic_isr, // LPUART8 (27)
    my_generic_isr, // LPI2C1 (28)
    my_generic_isr, // LPI2C2 (29)
    my_generic_isr, // LPI2C3 (30)
    my_generic_isr, // LPI2C4 (31)
    my_generic_isr, // LPSPI1 (32)
    my_generic_isr, // LPSPI2 (33)
    my_generic_isr, // LPSPI3 (34)
    my_generic_isr, // LPSPI4 (35)
    my_generic_isr, // FLEXCAN1 (36)
    my_generic_isr, // FLEXCAN2 (37)
    my_generic_isr, // CM7 (38)
    my_generic_isr, // KPP (39)
    my_generic_isr, // TSC_DIG (40)
    my_generic_isr, // GPR_IRQ (41)
    my_generic_isr, // LCDIF (42)
    my_generic_isr, // CSI (43)
    my_generic_isr, // PXP (44)
    my_generic_isr, // WDOG2 (45)
    my_generic_isr, // SNVS_HP_WRAPPER (46)
    my_generic_isr, // SNVS_HP_WRAPPER (47)
    my_generic_isr, // SNVS_HP_WRAPPER / SNVS_LP_WRAPPER (48)
    my_generic_isr, // CSU (49)
    my_generic_isr, // DCP (50)
    my_generic_isr, // DCP (51)
    my_generic_isr, // DCP (52)
    my_generic_isr, // TRNG (53)
    my_generic_isr, // Reserved (54)
    my_generic_isr, // BEE (55)
    my_generic_isr, // SAI1 (56)
    my_generic_isr, // SAI2 (57)
    my_generic_isr, // SAI3 (58)
    my_generic_isr, // SAI3 (59)
    my_generic_isr, // SPDIF (60)
    my_generic_isr, // PMU (61)
    my_generic_isr, // Reserved (62)
    my_generic_isr, // Temperature Monitor (63)
    my_generic_isr, // Temperature Monitor (64)
    my_generic_isr, // USB PHY (65)
    my_generic_isr, // USB PHY (66)
    my_generic_isr, // ADC1 (67)
    my_generic_isr, // ADC2 (68)
    my_generic_isr, // DCDC (69)
    my_generic_isr, // Reserved (70)
    my_generic_isr, // Reserved (71)
    my_generic_isr, // GPIO1 (72)
    my_generic_isr, // GPIO1 (73)
    my_generic_isr, // GPIO1 (74)
    my_generic_isr, // GPIO1 (75)
    my_generic_isr, // GPIO1 (76)
    my_generic_isr, // GPIO1 (77)
    my_generic_isr, // GPIO1 (78)
    my_generic_isr, // GPIO1 (79)
    my_generic_isr, // GPIO1_1 (80)
    my_generic_isr, // GPIO1_2 (81)
    my_generic_isr, // GPIO2_1 (82)
    my_generic_isr, // GPIO2_2 (83)
    my_generic_isr, // GPIO3_1 (84)
    my_generic_isr, // GPIO3_2 (85)
    my_generic_isr, // GPIO4_1 (86)
    my_generic_isr, // GPIO4_2 (87)
    my_generic_isr, // GPIO5_1 (88)
    my_generic_isr, // GPIO5_2 (89)
    my_generic_isr, // FLEXIO1 (90)
    my_generic_isr, // FLEXIO2 (91)
    my_generic_isr, // WDOG1 (92)
    my_generic_isr, // RTWDOG (93)
    my_generic_isr, // EWM (94)
    my_generic_isr, // CCM (95)
    my_generic_isr, // CCM (96)
    my_generic_isr, // GPC (97)
    my_generic_isr, // SRC (98)
    my_generic_isr, // Reserved (99)
    my_generic_isr, // GPT1 (100)
    my_generic_isr, // GPT2 (101)
    my_generic_isr, // FLEXPWM1 (102)
    my_generic_isr, // FLEXPWM1 (103)
    my_generic_isr, // FLEXPWM1 (104)
    my_generic_isr, // FLEXPWM1 (105)
    my_generic_isr, // FLEXPWM1 (106)
    my_generic_isr, // Reserved (107)
    my_generic_isr, // FLEXSPI (108)
    my_generic_isr, // SEMC (109)
    my_generic_isr, // USDHC1 (110)
    my_generic_isr, // USDHC2 (111)
    my_generic_isr, // USB (112)
    my_generic_isr, // USB (113)
    my_generic_isr, // ENET (114)
    my_generic_isr, // ENET (115)
    my_generic_isr, // XBAR1 (116)
    my_generic_isr, // XBAR1 (117)
    my_generic_isr, // ADC_ETC (118)
    my_generic_isr, // ADC_ETC (119)
    my_generic_isr, // ADC_ETC (120)
    my_generic_isr, // ADC_ETC (121)
    my_generic_isr, // PIT (122)
    my_generic_isr, // ACMP (123)
    my_generic_isr, // ACMP (124)
    my_generic_isr, // ACMP (125)
    my_generic_isr, // ACMP (126)
    my_generic_isr, // Reserved (127)
    my_generic_isr, // Reserved (128)
    my_generic_isr, // ENC1 (129)
    my_generic_isr, // ENC2 (130)
    my_generic_isr, // ENC3 (131)
    my_generic_isr, // ENC4 (132)
    my_generic_isr, // QTIMER1 (133)
    my_generic_isr, // QTIMER2 (134)
    my_generic_isr, // QTIMER3 (135)
    my_generic_isr, // QTIMER4 (136)
    my_generic_isr, // FLEXPWM2 (137)
    my_generic_isr, // FLEXPWM2 (138)
    my_generic_isr, // FLEXPWM2 (139)
    my_generic_isr, // FLEXPWM2 (140)
    my_generic_isr, // FLEXPWM2 (141)
    my_generic_isr, // FLEXPWM3 (142)
    my_generic_isr, // FLEXPWM3 (143)
    my_generic_isr, // FLEXPWM3 (144)
    my_generic_isr, // FLEXPWM3 (145)
    my_generic_isr, // FLEXPWM3 (146)
    my_generic_isr, // FLEXPWM4 (147)
    my_generic_isr, // FLEXPWM4 (148)
    my_generic_isr, // FLEXPWM4 (149)
    my_generic_isr, // FLEXPWM4 (150)
    my_generic_isr, // FLEXPWM4 (151)
    my_generic_isr, // Reserved (152)
    my_generic_isr, // Reserved (153)
    my_generic_isr, // Reserved (154)
    my_generic_isr, // Reserved (155)
    my_generic_isr, // Reserved (156)
    my_generic_isr, // Reserved (157)
    my_generic_isr, // Reserved (158)
    my_generic_isr, // Reserved (159)
];

// #[cfg_attr(all(target_arch = "arm", target_os = "none"), used)]
// pub static IRQS: [unsafe extern "C" fn(); 160] = [

pub unsafe fn init() {
    cortexm7::nvic::disable_all();
    cortexm7::nvic::clear_all_pending();

    tock_rt0::init_data(&mut _etext, &mut _srelocate, &mut _erelocate);
    tock_rt0::zero_bss(&mut _szero, &mut _ezero);


    cortexm::scb::set_vector_table_offset(
        &BASE_VECTORS as *const [unsafe extern "C" fn(); 16] as *const (),
    );
    
    // panic!("{:?} {:?}", &IRQS as *const [unsafe extern "C" fn(); 160] as *const (),&BASE_VECTORS as *const [unsafe extern "C" fn(); 16] as *const ());
    // hprintln!("{:?}", &IRQS as *const [unsafe extern "C" fn(); 160] as *const ()).unwrap();
    // hprintln!("{:?}", &BASE_VECTORS as *const [unsafe extern "C" fn(); 16] as *const ()).unwrap();
    // for i in 0..160 {
    //     INTERRUPT_BASE.interrupt[i].modify(INTERRUPT::VAL.val(my_generic_isr as unsafe extern "C" fn() as u32));
    // }
    ccm::CCM.set_low_power_mode();
}

pub unsafe fn magick_changer() {
     cortexm::scb::set_vector_table_offset(
        &NEW_IRQS as *const [unsafe extern "C" fn(); 176] as *const (),
    );
}