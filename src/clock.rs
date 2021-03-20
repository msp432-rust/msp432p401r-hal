//! HAL library for CS (Clock System) - MSP432P401R

/*
+--------+-----------------------+---------------+-------------------------------------------------+
| Clock  | Default Clock Source  | Default Clock | Description                                     |                                                     |
+========+=======================+===============+=================================================+
| MCLK   | DCO                   | 3 MHz         | Master Clock - Sources CPU and peripherals      |
+--------------+-----------------+---------------+-------------------------------------------------+
| HSMCLK | DCO                   | 3 MHz         | Subsystem Master Clock - Sources and peripherals|
+--------------+-----------------+---------------+-------------------------------------------------+
| SMCLK  | DCO                   | 3 MHz         | Low-speed Subsystem Master Clock                |
+--------------+-----------------+---------------+-------------------------------------------------+
| ACLK   | LFXT (or REFO)        | 32.768 kHz    | Auxiliary clock - Sources  and  peripherals     |
+--------------+-----------------+---------------+-------------------------------------------------+
| BCLK   | LFXT (or REFO)        | 32.768 kHz    | Low-speed backup domain clock - LPM peripherals |
+--------+-----------------------+---------------+-------------------------------------------------+

Clock Sources:
• LFXTCLK: Low-frequency oscillator (LFXT) that can be used either with low-frequency 32768-Hz crys-
tals or external clock sources in the 32-kHz or below range in bypass mode.
• HFXTCLK: High-frequency oscillator (HFXT) that can be used with standard crystals or resonators in
the 1 MHz to 48-MHz range. When in bypass mode, HFXTCLK can be driven with an external square wave.
• DCOCLK: Internal digitally controlled oscillator (DCO) with programmable frequencies: 3-MHz default
(Prog. Freq. 1,5  3  6  12  24  48 MHz)
• VLOCLK: Internal very-low-power low-frequency oscillator (VLO) with 9.4-kHz typical frequency
• REFOCLK : Internal, low-power low-frequency oscillator (REFO) with selectable 32.768-kHz or 128-kHz
• MODCLK: Internal low-power oscillator with 25-MHz typical frequency
• SYSOSC: Internal oscillator with 5-MHz typical frequency

ACLK: Auxiliary clock
ACLK is software selectable as LFXTCLK, VLOCLK, or REFOCLK.
ACLK can be divided by 1, 2, 4, 8, 16, 32, 64, or 128.
ACLK is restricted to maximum frequency of operation of 128 kHz.

MCLK: Master clock
MCLK is software selectable as LFXTCLK, VLOCLK, REFOCLK, DCOCLK, MODCLK, or HFXTCLK.
MCLK can be divided by 1, 2, 4, 8, 16, 32, 64, or 128.

HSMCLK: Subsystem master clock.
HSMCLK is software selectable as LFXTCLK, VLOCLK, REFOCLK, DCOCLK, MODCLK, HFXTCLK.
HSMCLK can be divided by 1, 2, 4, 8, 16, 32, 64, or 128.

SMCLK: Low-speed subsystem master clock.
SMCLK uses the HSMCLK clock resource selection for its clock resource.
SMCLK can be divided independently from HSMCLK by 1, 2, 4, 8, 16, 32, 64, or 128.
SMCLK is limited in frequency to half of the rated maximum frequency of HSMCLK.

BCLK: Low-speed backup domain clock.
BCLK is software selectable as LFXTCLK and REFOCLK and is used primarily in the backup domain.
BCLK is restricted to a maximum frequency of 32.768 kHz
*/

use cortex_m::interrupt;

use crate::time::Hertz;
use pac::cs::csctl0::DCORSEL_A;
use pac::cs::csctl2::HFXTFREQ_A;
pub use pac::cs::csctl1::{SELS_A, SELA_A, SELM_A, DIVM_A as MPrescaler, DIVS_A as SMPrescaler};
use pac::cs::csclken::REFOFSEL_A;
use pac::CS;

pub trait CsExt {
    fn constrain(self) -> ClockConfig<NoClockDefined, NoClockDefined>;
}

impl CsExt for CS {
    fn constrain(self) -> ClockConfig<NoClockDefined, NoClockDefined> {
        ClockConfig::new(self)
    }
}

/// Typestates for `ClockConfig`
pub struct NoClockDefined;

pub struct MclkDefined(MasterClock);

pub struct SmclkDefined(SMPrescaler);

pub const MODCLK: u32 = 25_000_000;
pub const VLOCLK: u32 = 9_400;
pub const LFXTCLK: u32 = 32_768;
pub const HFXTMINCLK: u32 = 1_000_000;
pub const HFXTMAXCLK: u32 = 48_000_000;

#[derive(Debug, Clone, Copy)]
pub enum HighFrequencyCrystal {
    _1_4MHz,
    _4_8MHz,
    _8_16MHz,
    _16_24MHz,
    _24_32MHz,
    _32_40MHz,
    _40_48MHz,
}

impl HighFrequencyCrystal {
    const fn hfxtsel(&self) -> HFXTFREQ_A {
        match *self {
            HighFrequencyCrystal::_1_4MHz => HFXTFREQ_A::HFXTFREQ_0,
            HighFrequencyCrystal::_4_8MHz => HFXTFREQ_A::HFXTFREQ_1,
            HighFrequencyCrystal::_8_16MHz => HFXTFREQ_A::HFXTFREQ_2,
            HighFrequencyCrystal::_16_24MHz => HFXTFREQ_A::HFXTFREQ_3,
            HighFrequencyCrystal::_24_32MHz => HFXTFREQ_A::HFXTFREQ_4,
            HighFrequencyCrystal::_32_40MHz => HFXTFREQ_A::HFXTFREQ_5,
            HighFrequencyCrystal::_40_48MHz => HFXTFREQ_A::HFXTFREQ_6,
        }
    }

    pub const fn freq(&self) -> u32 {
        match *self {
            HighFrequencyCrystal::_1_4MHz => 4_000_000,
            HighFrequencyCrystal::_4_8MHz => 8_000_000,
            HighFrequencyCrystal::_8_16MHz => 16_000_000,
            HighFrequencyCrystal::_16_24MHz => 24_000_000,
            HighFrequencyCrystal::_24_32MHz => 32_000_000,
            HighFrequencyCrystal::_32_40MHz => 40_000_000,
            HighFrequencyCrystal::_40_48MHz => 48_000_000,
        }
    }
}

/// Default: 32.768 KHz
#[derive(Debug, Clone, Copy)]
pub enum REFOFrequency {
    _32kHz,
    _128kHz,
}

impl REFOFrequency {
    fn refofsel(&self) -> REFOFSEL_A {
        match *self {
            REFOFrequency::_32kHz => REFOFSEL_A::REFOFSEL_0,
            REFOFrequency::_128kHz => REFOFSEL_A::REFOFSEL_1,
        }
    }

    pub fn freq(&self) -> u32 {
        match *self {
            REFOFrequency::_32kHz => 32_768,
            REFOFrequency::_128kHz => 128_000,
        }
    }
}

/// Default: 3 MHz
#[derive(Debug, Clone, Copy)]
pub enum DCOFrequency {
    _1500kHz,
    _3MHz,
    _6MHz,
    _12MHz,
    _24MHz,
    _48MHz,
}

impl DCOFrequency {
    fn frequecy_range(&self) -> DCORSEL_A {
        match *self {
            DCOFrequency::_1500kHz => DCORSEL_A::DCORSEL_0,
            DCOFrequency::_3MHz => DCORSEL_A::DCORSEL_1,
            DCOFrequency::_6MHz => DCORSEL_A::DCORSEL_2,
            DCOFrequency::_12MHz => DCORSEL_A::DCORSEL_3,
            DCOFrequency::_24MHz => DCORSEL_A::DCORSEL_4,
            DCOFrequency::_48MHz => DCORSEL_A::DCORSEL_5,
        }
    }

    pub fn freq(&self) -> u32 {
        match *self {
            DCOFrequency::_1500kHz => 1_500_000,
            DCOFrequency::_3MHz => 3_000_000,
            DCOFrequency::_6MHz => 6_000_000,
            DCOFrequency::_12MHz => 12_000_000,
            DCOFrequency::_24MHz => 24_000_000,
            DCOFrequency::_48MHz => 48_000_000,
        }
    }
}

#[derive(Debug)]
enum MasterClock {
    VLOCLK,
    MODCLK,
    LFXTCLK,
    REFOCLK(REFOFrequency),
    HFXTCLK(HighFrequencyCrystal),
    DCOCLK(DCOFrequency),
}

impl MasterClock {
    fn freq(&self) -> u32 {
        match *self {
            MasterClock::VLOCLK => VLOCLK as u32,
            MasterClock::MODCLK => MODCLK as u32,
            MasterClock::LFXTCLK => LFXTCLK as u32,
            MasterClock::REFOCLK(sel) => sel.freq(),
            MasterClock::HFXTCLK(sel) => sel.freq(),
            MasterClock::DCOCLK(sel) => sel.freq(),
        }
    }

    fn selm(&self) -> SELM_A {
        match *self {
            MasterClock::LFXTCLK => SELM_A::SELM_0,
            MasterClock::VLOCLK => SELM_A::SELM_1,
            MasterClock::REFOCLK(_) => SELM_A::SELM_2,
            MasterClock::DCOCLK(_) => SELM_A::SELM_3,
            MasterClock::MODCLK => SELM_A::SELM_4,
            MasterClock::HFXTCLK(_) => SELM_A::SELM_5,
        }
    }
}

#[derive(Debug)]
enum HSMclkSel {
    Vloclk,
    Modclk,
    Lfxtclk,
    Refoclk(REFOFrequency),
    Hfxtclk(HighFrequencyCrystal),
    Dcoclk(DCOFrequency),
}

impl HSMclkSel {
    fn freq(&self) -> u32 {
        match *self {
            HSMclkSel::Vloclk => VLOCLK as u32,
            HSMclkSel::Modclk => MODCLK as u32,
            HSMclkSel::Lfxtclk => LFXTCLK as u32,
            HSMclkSel::Refoclk(sel) => sel.freq(),
            HSMclkSel::Hfxtclk(sel) => sel.freq(),
            HSMclkSel::Dcoclk(sel) => sel.freq(),
        }
    }

    fn sels(&self) -> SELS_A {
        match *self {
            HSMclkSel::Lfxtclk => SELS_A::SELS_0,
            HSMclkSel::Vloclk => SELS_A::SELS_1,
            HSMclkSel::Refoclk(_) => SELS_A::SELS_2,
            HSMclkSel::Dcoclk(_) => SELS_A::SELS_3,
            HSMclkSel::Modclk => SELS_A::SELS_4,
            HSMclkSel::Hfxtclk(_) => SELS_A::SELS_5,
        }
    }
}

#[derive(Clone, Copy)]
enum AclkSel {
    Lfxtclk,
    Vloclk,
    Refoclk(REFOFrequency),
}

impl AclkSel {
    fn sela(&self) -> SELA_A {
        match *self {
            AclkSel::Lfxtclk => SELA_A::SELA_0,
            AclkSel::Vloclk => SELA_A::SELA_1,
            AclkSel::Refoclk(_) => SELA_A::SELA_2,
        }
    }

    fn freq(&self) -> u32 {
        match *self {
            AclkSel::Vloclk => VLOCLK as u32,
            AclkSel::Refoclk(sel) => sel.freq(),
            AclkSel::Lfxtclk => LFXTCLK as u32,
        }
    }
}

#[doc(hidden)]
pub trait SmclkState {
    fn div(&self) -> Option<SMPrescaler>;
}

impl SmclkState for SmclkDefined {
    fn div(&self) -> Option<SMPrescaler> {
        Some(*&self.0)
    }
}

/// Builder object that configures system clocks
/// Can only commit configurations to hardware if both MCLK (HSMCLK) and SMCLK settings have been
/// configured. ACLK and BCLK configurations are optional, with its default source being REFOCLK.
pub struct ClockConfig<MCLK, SMCLK> {
    cs: CS,
    mclk: MCLK,
    mclk_div: MPrescaler,
    aclk_sel: AclkSel,
    smclk_sel: HSMclkSel,
    smclk: SMCLK,
}

macro_rules! make_clkconf {
    ($conf:expr, $mclk:expr, $smclk:expr) => {
        ClockConfig {
            cs: $conf.cs,
            mclk: $mclk,
            mclk_div: $conf.mclk_div,
            aclk_sel: $conf.aclk_sel,
            smclk_sel: $conf.smclk_sel,
            smclk: $smclk,
        }
    };
}

impl ClockConfig<NoClockDefined, NoClockDefined> {
    /// Converts CS into a fresh, unconfigured clock builder object
    fn new(cs: CS) -> Self {
        ClockConfig {
            cs,
            smclk: NoClockDefined,
            mclk: NoClockDefined,
            mclk_div: MPrescaler::DIVM_0,
            aclk_sel: AclkSel::Refoclk(REFOFrequency::_32kHz),
            smclk_sel: HSMclkSel::Dcoclk(DCOFrequency::_3MHz),
        }
    }
}

impl<MCLK, SMCLK> ClockConfig<MCLK, SMCLK> {
    pub fn aclk_lfcrystalsource_selection(mut self) -> Self {
        self.aclk_sel = AclkSel::Lfxtclk;
        self
    }

    pub fn aclk_refosource_selection(mut self, target_freq: REFOFrequency) -> Self {
        self.aclk_sel = AclkSel::Refoclk(target_freq);
        self
    }

    pub fn aclk_vlosource_selection(mut self) -> Self {
        self.aclk_sel = AclkSel::Vloclk;
        self
    }

    /// Select REFOCLK for MCLK and set the MCLK divider. Frequency is `REFOCLK / mclk_div` Hz.
    pub fn mclk_refosource_selection(self, target_freq: REFOFrequency, mclk_div: MPrescaler) -> ClockConfig<MclkDefined, SMCLK> {
        let smclk_sel = HSMclkSel::Refoclk(target_freq);

        ClockConfig {
            mclk_div,
            smclk_sel,
            ..make_clkconf!(self, MclkDefined(MasterClock::REFOCLK(target_freq)), self.smclk)
        }
    }

    /// Select LFXTCLK for MCLK and set the MCLK divider. Frequency is `LFXTCLK / mclk_div` Hz.
    pub fn mclk_lfcrystalsource_selection(self, mclk_div: MPrescaler) -> ClockConfig<MclkDefined, SMCLK> {
        let smclk_sel = HSMclkSel::Lfxtclk;

        ClockConfig {
            mclk_div,
            smclk_sel,
            ..make_clkconf!(self, MclkDefined(MasterClock::LFXTCLK), self.smclk)
        }
    }

    /// Select MODCLK for MCLK and set the MCLK divider. Frequency is `MODCLK / mclk_div` Hz.
    pub fn mclk_modsource_selection(self, mclk_div: MPrescaler) -> ClockConfig<MclkDefined, SMCLK> {
        let smclk_sel = HSMclkSel::Modclk;

        ClockConfig {
            mclk_div,
            smclk_sel,
            ..make_clkconf!(self, MclkDefined(MasterClock::MODCLK), self.smclk)
        }
    }

    /// Select VLOCLK for MCLK and set the MCLK divider. Frequency is `VLO / mclk_div` Hz.
    pub fn mclk_vlosource_selection(self, mclk_div: MPrescaler) -> ClockConfig<MclkDefined, SMCLK> {
        let smclk_sel = HSMclkSel::Vloclk;

        ClockConfig {
            mclk_div,
            smclk_sel,
            ..make_clkconf!(self, MclkDefined(MasterClock::VLOCLK), self.smclk)
        }
    }

    /// Select HFXTCLK for MCLK and set the MCLK divider. Frequency is `HFXTCLK / mclk_div` Hz.
    pub fn mclk_hfcrystalsource_selection(self, target_freq: HighFrequencyCrystal, mclk_div: MPrescaler) -> ClockConfig<MclkDefined, SMCLK> {
        let smclk_sel = HSMclkSel::Hfxtclk(target_freq);

        ClockConfig {
            mclk_div,
            smclk_sel,
            ..make_clkconf!(self, MclkDefined(MasterClock::HFXTCLK(target_freq)), self.smclk)
        }
    }

    /// Select DCOCLK for MCLK with FLL for stabilization. Frequency is `target_freq / mclk_div` Hz.
    pub fn mclk_dcosource_selection(self, target_freq: DCOFrequency, mclk_div: MPrescaler) -> ClockConfig<MclkDefined, SMCLK> {
        let smclk_sel = HSMclkSel::Dcoclk(target_freq);

        ClockConfig {
            mclk_div,
            smclk_sel,
            ..make_clkconf!(self, MclkDefined(MasterClock::DCOCLK(target_freq)), self.smclk)
        }
    }

    /// Enable SMCLK and set SMCLK divider, which divides the MCLK frequency
    pub fn smclk_prescaler(self, div: SMPrescaler) -> ClockConfig<MCLK, SmclkDefined> {
        make_clkconf!(self, self.mclk, SmclkDefined(div))
    }
}

impl<SMCLK: SmclkState> ClockConfig<MclkDefined, SMCLK> {
    fn configure_dco_fll(&self) {
        if let MasterClock::DCOCLK(target_freq) = self.mclk.0 {
            self.cs.csctl0.write(|w| { w.dcorsel().variant(target_freq.frequecy_range()) });

            for _n in 1..50 {
                unsafe { llvm_asm!("NOP") };
            }
        };
    }

    fn configure_hfxt(&self) {
        if let MasterClock::HFXTCLK(target_freq) = self.mclk.0 {
            self.cs.csctl2.write(|w| { w.hfxtfreq().variant(target_freq.hfxtsel()) });

            for _n in 1..50 {
                unsafe { llvm_asm!("NOP") };
            }
        };
    }

    fn configure_refo(&self) {
        if let MasterClock::REFOCLK(target_freq) = self.mclk.0 {
            self.cs.csclken.write(|w| { w.refofsel().variant(target_freq.refofsel()) })
        };

        if let AclkSel::Refoclk(target_freq) = self.aclk_sel {
            self.cs.csclken.write(|w| { w.refofsel().variant(target_freq.refofsel()) })
        };
    }

    fn configure_cs(&self) {
        const CS_STAT: u8 = 0b00011111;
        const CS_MASK_ACLK: u32 = 0xF8FFF8FF;
        const CS_MASK_MCLK: u32 = 0xFFF8FFF8;
        const CS_MASK_SMCLK: u32 = 0x8FFFFF8F;

        // Configure clock selector and divisors
        self.wait_clk(CS_STAT);

        // ACLK SEL
        self.cs.csctl1.modify(|r, w| unsafe {
            w.bits(r.bits() & CS_MASK_ACLK)
                .sela().variant(self.aclk_sel.sela())
        });

        self.wait_clk(CS_STAT);

        // MCLK SEL | MCLK DIV
        self.cs.csctl1.modify(|r, w| unsafe {
            w.bits(r.bits() & CS_MASK_MCLK)
                .selm().variant(self.mclk.0.selm())
                .divm().variant(self.mclk_div)
        });

        self.wait_clk(CS_STAT);

        // SMCLK SEL | SMCLK DIV
        self.cs.csctl1.modify(|r, w| unsafe {
            w.bits(r.bits() & CS_MASK_SMCLK)
                .sels().variant(self.smclk_sel.sels());
            match self.smclk.div() {
                Some(div) => w.divs().variant(div),
                None => w.divs().variant(SMPrescaler::DIVS_0),
            }
        });

        self.wait_clk(CS_STAT);
    }

    fn cs_key(&self, keylock: bool) {
        const CSKEY: u32 = 0x695A;

        if keylock {
            self.cs.cskey.write(|w| unsafe { w.bits(CSKEY) });
        } else {
            self.cs.cskey.write(|w| unsafe { w.bits(!CSKEY) });
        }
    }

    fn wait_clk(&self, flag: u8) {
        for _n in 1..50 {
            unsafe { llvm_asm!("NOP") };
        }

        while ((self.cs.csstat.read().bits() >> 24) as u8 & flag) != flag {
            unsafe { llvm_asm!("NOP") };
        };
    }
}

impl ClockConfig<MclkDefined, SmclkDefined> {
    /// Apply clock configuration to hardware and return clock objects
    pub fn freeze(self) -> Clocks {
        interrupt::free(|_| {
            self.cs_key(true);

            self.configure_dco_fll();
            self.configure_hfxt();
            self.configure_refo();
            self.configure_cs();

            self.cs_key(false);
        });

        let aclk_freq: u32 = self.aclk_sel.freq();
        let mclk_freq = self.mclk.0.freq() >> self.mclk_div as u32;
        let hsmclk_freq: u32 = self.smclk_sel.freq();
        let smclk_freq: u32 = self.smclk_sel.freq() / self.smclk.0 as u32;

        let clocks = Clocks {
            aclk: Hertz(aclk_freq),
            mclk: Hertz(mclk_freq),
            hsmclk: Hertz(hsmclk_freq),
            smclk: Hertz(smclk_freq),
            bclk: Hertz(aclk_freq),
        };

        clocks
    }
}

/// Frozen clock frequencies
/// This value holds the current clocks frequencies
#[derive(Clone, Copy)]
pub struct Clocks {
    aclk: Hertz,
    mclk: Hertz,
    hsmclk: Hertz,
    smclk: Hertz,
    bclk: Hertz,
}

impl Clocks {
    pub fn aux_clock(&self) -> Hertz {
        self.aclk
    }

    pub fn master_clock(&self) -> Hertz {
        self.mclk
    }

    pub fn subsystem_master_clock(&self) -> Hertz {
        self.hsmclk
    }

    pub fn low_frequency_subsystem_master_clock(&self) -> Hertz {
        self.smclk
    }

    pub fn backup_clock(&self) -> Hertz {
        self.bclk
    }
}
