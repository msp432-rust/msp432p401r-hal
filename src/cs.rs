//! HAL library for CS (Clock Source) - MSP432P401R

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

use core::cmp;
use crate::time::Hertz;
use pac::cs::csctl0::DCORSEL_A;
use pac::cs::csctl2::HFXTFREQ_A;
pub use pac::cs::csctl1::{SELM_A, DIVM_A, DIVS_A as SmclkDiv};
use pac::cs::csclken::REFOFSEL_A;

/// Typestate for `ClockConfig` that represents unconfigured clocks
pub struct NoClockDefined;
/// Typestate for `ClockConfig` that represents a configured MCLK
pub struct MclkDefined(MclkSel);
/// Typestate for `ClockConfig` that represents a configured SMCLK
pub struct SmclkDefined(SmclkDiv);

/// MODCLK frequency
pub const MODCLK: u32 = 25_000_000;
/// VLOCLK frequency
pub const VLOCLK: u32 = 9_400;
/// LFXTCLK frequency
pub const LFXTCLK: u32 = 32_768;

///Selectable HFXTCLK frequencies
#[derive(Clone, Copy)]
pub enum HfxtFreqSel {
    /// 1 MHz
    _1MHz = 1_000_000,
    /// 4 MHz
    _4MHz = 4_000_000,
    /// 8 MHz
    _8MHz = 8_000_000,
    /// 16 MHz
    _16MHz = 16_000_000,
    /// 24 MHz
    _24MHz = 24_000_000,
    /// 32 MHz
    _32MHz = 32_000_000,
    /// 40 MHz
    _40MHz = 40_000_000,
    /// 48 MHz
    _48MHz = 48_000_000,
}

impl HfxtFreqSel {
    #[inline(always)]
    fn hfxtsel(self) -> HFXTFREQ_A {
        match self {
            HfxtFreqSel::_1MHz ..= HfxtFreqSel::_4MHz => HFXTFREQ_A::HFXTFREQ_0,
            HfxtFreqSel::_4MHz ..= HfxtFreqSel::_8MHz => HFXTFREQ_A::HFXTFREQ_1,
            HfxtFreqSel::_8MHz ..= HfxtFreqSel::_16MHz => HFXTFREQ_A::HFXTFREQ_2,
            HfxtFreqSel::_16MHz ..= HfxtFreqSel::_24MHz => HFXTFREQ_A::HFXTFREQ_3,
            HfxtFreqSel::_24MHz ..= HfxtFreqSel::_32MHz => HFXTFREQ_A::HFXTFREQ_4,
            HfxtFreqSel::_32MHz ..= HfxtFreqSel::_40MHz => HFXTFREQ_A::HFXTFREQ_5,
            HfxtFreqSel::_40MHz ..= HfxtFreqSel::_48MHz => HFXTFREQ_A::HFXTFREQ_6,
            _=>HFXTFREQ_A::HFXTFREQ_6,
        }
    }

    /// Numerical frequency
    #[inline]
    pub fn freq(self) -> u32 {
        match self {
            HfxtFreqSel::_1MHz ..= HfxtFreqSel::_48MHz => self.0 as u32,
            _=> _48MHz as u32,
        }
    }
}

/// Selectable REFOCLK frequencies
/// Default: 32.768 KHz
#[derive(Clone, Copy)]
pub enum RefoclkFreqSel {
    // 32.738 KHz
    _32_768 = 32_768,
    // 128 KHz
    _128_000 = 128_000,
}

impl RefoclkFreqSel {
    #[inline(always)]
    fn refofsel(self) -> REFOFSEL_A {
        match self {
            RefoclkFreqSel::_32_768 => REFOFSEL_A::REFOFSEL_0,
            RefoclkFreqSel::_128_000 => REFOFSEL_A::REFOFSEL_1,
            _ => REFOFSEL_A::REFOFSEL_0,
        }
    }

    /// Numerical frequency
    #[inline]
    pub fn freq(self) -> u32 {
        match self {
            RefoclkFreqSel::_32_768 => self.0 as u32,
            RefoclkFreqSel::_128_000 => self.0 as u32,
            _=> _32_768 as u32,
        }
    }
}

/// Selectable DCOCLK frequencies when using factory trim settings.
/// Actual frequencies may be slightly higher.
#[derive(Clone, Copy)]
pub enum DcoclkFreqSel {
    /// 1,5 MHz
    _1_5MHz = 1_500_000,
    /// 3 MHz
    _3MHz = 3_000_000,
    /// 6 MHz
    _6MHz = 6_000_000,
    /// 12 MHz
    _12MHz = 12_000_000,
    /// 24 MHz
    _24MHz = 24_000_000,
    /// 48 MHz
    _48MHz = 48_000_000,
}

impl DcoclkFreqSel {
    #[inline(always)]
    fn dcorsel(self) -> DCORSEL_A {
        match self {
            DcoclkFreqSel::_1_5MHz => DCORSEL_A::DCORSEL_0,
            DcoclkFreqSel::_3MHz => DCORSEL_A::DCORSEL_1,
            DcoclkFreqSel::_6MHz => DCORSEL_A::DCORSEL_2,
            DcoclkFreqSel::_12MHz => DCORSEL_A::DCORSEL_3,
            DcoclkFreqSel::_24MHz => DCORSEL_A::DCORSEL_4,
            DcoclkFreqSel::_48MHz => DCORSEL_A::DCORSEL_5,
            _=> DCORSEL_A::DCORSEL_5,
        }
    }

    /// Numerical frequency
    #[inline]
    pub fn freq(self) -> u32 {
        match self {
            DcoclkFreqSel::_1_5MHz => self.0 as u32,
            DcoclkFreqSel::_3MHz => self.0 as u32,
            DcoclkFreqSel::_6MHz => self.0 as u32,
            DcoclkFreqSel::_12MHz => self.0 as u32,
            DcoclkFreqSel::_24MHz => self.0 as u32,
            DcoclkFreqSel::_48MHz => self.0 as u32,
            _=> _48MHz as u32,
        }
    }
}

// ***** Mclk *****
enum MclkSel {
    Vloclk,
    Modclk,
    Lfxtclk,
    Refoclk(RefoclkFreqSel),
    Hfxtclk(HftxtclkFreqSel),
    Dcoclk(DcoclkFreqSel),
}

impl MclkSel {
    #[inline]
    fn freq(&self) -> u32 {
        match self {
            MclkSel::Vloclk => VLOCLK as u32,
            MclkSel::Modclk => MODCLK as u32,
            MclkSel::Lfxtclk => LFXTCLK as u32,
            MclkSel::Refoclk(sel) => sel.freq(),
            MclkSel::Hfxtclk(sel) => sel.freq(),
            MclkSel::Dcoclk(sel) => sel.freq(),
        }
    }

    #[inline(always)]
    fn selm(&self) -> SELM_A {
        match self {
            MclkSel::Lfxtclk => SELM_A::SELM_0,
            MclkSel::Vloclk => SELM_A::SELM_1,
            MclkSel::Refoclk(_) => SELM_A::SELM_2,
            MclkSel::Dcoclk(_) => SELM_A::SELM_3,
            MclkSel::Modclk => SELM_A::SELM_4,
            MclkSel::Hfxtclk(_) => SELM_A::SELM_5,
            _=> SELM_A::SELM_3,
        }
    }
}

// ***** Aclk *****
#[derive(Clone, Copy)]
enum AclkSel {
    Lfxtclk,
    Vloclk,
    Refoclk(RefoclkFreqSel),
}

impl AclkSel {
    #[inline(always)]
    fn sela(self) -> SELA_A {
        match self {
            AclkSel::Lfxtclk => SELA_A::SELA_0,
            AclkSel::Vloclk => SELA_A::SELA_1,
            AclkSel::Refoclk(_) => SELA_A::SELA_2,
            _=> SELA_A::SELA_2,
        }
    }

    #[inline(always)]
    fn freq(self) -> u32 {
        match self {
            AclkSel::Vloclk => VLOCLK as u32,
            AclkSel::Refoclk(sel) => sel.freq(),
            AclkSel::Lfxtclk => LFXTCLK as u32,
        }
    }
}

// Using SmclkState as a trait bound outside the HAL will never be useful, since we only configure
// the clock once, so just keep it hidden
#[doc(hidden)]
pub trait SmclkState {
    fn div(&self) -> Option<SmclkDiv>;
}

impl SmclkState for SmclkDefined {
    #[inline(always)]
    fn div(&self) -> Option<SmclkDiv> {
        Some(self.0)
    }
}

/// Builder object that configures system clocks
/// Can only commit configurations to hardware if both MCLK (HSMCLK) and SMCLK settings have been
/// configured. ACLK and BCLK configurations are optional, with its default source being REFOCLK.
pub struct ClockConfig<MCLK, SMCLK> {
    periph: pac::CS,
    mclk: MCLK,
    mclk_div: DIVM_A,
    aclk_sel: AclkSel,
    smclk: SMCLK,
}

macro_rules! make_clkconf {
    ($conf:expr, $mclk:expr, $smclk:expr) => {
        ClockConfig {
            periph: $conf.periph,
            mclk: $mclk,
            mclk_div: $conf.mclk_div,
            aclk_sel: $conf.aclk_sel,
            smclk: $smclk,
        }
    };
}

impl ClockConfig<NoClockDefined, NoClockDefined> {
    /// Converts CS into a fresh, unconfigured clock builder object
    pub fn new(cs: pac::CS) -> Self {
        ClockConfig {
            periph: cs,
            smclk: NoClockDefined,
            mclk: NoClockDefined,
            mclk_div: DIVM_A::DIVM_0,
            aclk_sel: AclkSel::Refoclk(_32_768),
        }
    }
}

impl<MCLK, SMCLK> ClockConfig<MCLK, SMCLK> {
    /// Select LFXTCLK for ACLK
    #[inline]
    pub fn aclk_lfxtclk(mut self) -> Self {
        self.aclk_sel = AclkSel::Lfxtclk;
        self
    }

    /// Select REFOCLK for ACLK
    #[inline]
    pub fn aclk_refoclk(mut self, target_freq: RefoclkFreqSel) -> Self {
        self.aclk_sel = AclkSel::Refoclk(target_freq);
        self
    }

    /// Select VLOCLK for ACLK
    #[inline]
    pub fn aclk_vloclk(mut self) -> Self {
        self.aclk_sel = AclkSel::Vloclk;
        self
    }

    /// Select REFOCLK for MCLK and set the MCLK divider. Frequency is `REFOCLK / mclk_div` Hz.
    #[inline]
    pub fn mclk_refoclk(self, target_freq: RefoclkFreqSel, mclk_div: DIVM_A) -> ClockConfig<MclkDefined, SMCLK> {
        ClockConfig {
            mclk_div,
            ..make_clkconf!(self, MclkDefined(MclkSel::Refoclk(target_freq)), self.smclk)
        }
    }

    /// Select LFXTCLK for MCLK and set the MCLK divider. Frequency is `LFXTCLK / mclk_div` Hz.
    #[inline]
    pub fn mclk_lfxtclk(self, mclk_div: DIVM_A) -> ClockConfig<MclkDefined, SMCLK> {
        ClockConfig {
            mclk_div,
            ..make_clkconf!(self, MclkDefined(MclkSel::Lfxtclk), self.smclk)
        }
    }

    /// Select MODCLK for MCLK and set the MCLK divider. Frequency is `MODCLK / mclk_div` Hz.
    #[inline]
    pub fn mclk_modclk(self, mclk_div: DIVM_A) -> ClockConfig<MclkDefined, SMCLK> {
        ClockConfig {
            mclk_div,
            ..make_clkconf!(self, MclkDefined(MclkSel::Modclk), self.smclk)
        }
    }

    /// Select VLOCLK for MCLK and set the MCLK divider. Frequency is `VLO / mclk_div` Hz.
    #[inline]
    pub fn mclk_vloclk(self, mclk_div: DIVM_A) -> ClockConfig<MclkDefined, SMCLK> {
        ClockConfig {
            mclk_div,
            ..make_clkconf!(self, MclkDefined(MclkSel::Vloclk), self.smclk)
        }
    }

    /// Select HFXTCLK for MCLK and set the MCLK divider. Frequency is `HFXTCLK / mclk_div` Hz.
    #[inline]
    pub fn mclk_hfxtclk(self, target_freq: HfxtclkFreqSel, mclk_div: DIVM_A) -> ClockConfig<MclkDefined, SMCLK> {
        ClockConfig {
            mclk_div,
            ..make_clkconf!(self, MclkDefined(MclkSel::Hfxtclk(target_freq)), self.smclk)
        }
    }

    /// Select DCOCLK for MCLK with FLL for stabilization. Frequency is `target_freq / mclk_div` Hz.
    /// This setting selects the default factory trim for DCO trimming and performs no extra
    /// calibration, so only a select few frequency targets can be selected.
    #[inline]
    pub fn mclk_dcoclk(self, target_freq: DcoclkFreqSel, mclk_div: DIVM_A) -> ClockConfig<MclkDefined, SMCLK> {
        ClockConfig {
            mclk_div,
            ..make_clkconf!(self, MclkDefined(MclkSel::Dcoclk(target_freq)), self.smclk)
        }
    }

    /// Enable SMCLK and set SMCLK divider, which divides the MCLK frequency
    #[inline]
    pub fn smclk_div(self, div: SmclkDiv) -> ClockConfig<MCLK, SmclkDefined> {
        make_clkconf!(self, self.mclk, SmclkDefined(div))
    }
}

impl<SMCLK: SmclkState> ClockConfig<MclkDefined, SMCLK> {

    #[inline(always)]
    fn cs_key(&self, keylock: bool) {

        /// CSKEY
        const CSKEY: u16 = 0x695A;

        if keylock{
            self.periph.cskey.write(|w| unsafe { w.bits(CSKEY) });
        } else {
            self.periph.cskey.write(|w| unsafe { w.bits(!CSKEY) });
        }
    }

    #[inline(always)]
    fn wait_clk(&self, flag: u8) {
        while ((self.csstat.read().bits >> 24) as u8 & flag) != flag {}
    }

    #[inline]
    fn configure_dco_fll(&self) {
        // Run DCO configuration
        if let MclkSel::Dcoclk(target_freq) = self.mclk.0 {
            self.periph.csctl0.write(|w| unsafe { w.dcorsel().variant(target_freq.dcorsel()) })
            };

            unsafe {
                asm!("nop");
                asm!("nop");
                asm!("nop");
            }
        }

    #[inline]
    fn configure_hfxt(&self) {
        // Run HFXT configuration
        if let MclkSel::Hfxtclk(target_freq) = self.mclk.0 {
            self.periph.csctl2.write(|w| unsafe { w.hfxtsel().variant(target_freq.hfxtsel()) })
        };

        unsafe {
            asm!("nop");
            asm!("nop");
            asm!("nop");
        }
    }

    #[inline]
    fn configure_refo(&self) {
        // Run REFO configuration
        if let MclkSel::Refoclk(target_freq) = self.mclk.0 {
            self.periph.csclken.write(|w| unsafe { w.refosel().variant(target_freq.refosel()) })
        };

        if let AclkSel::Refoclk(target_freq) = self.mclk.0 {
            self.periph.csclken.write(|w| unsafe { w.refosel().variant(target_freq.refosel()) })
        };

        unsafe {
            asm!("nop");
            asm!("nop");
            asm!("nop");
        }
    }

    #[inline]
    fn configure_cs(&self) {
        // Configure clock selector and divisors
        self.periph.csctl1.write(|w| {
             w.sela()                           // ACLK SEL
                .variant(self.aclk_sel.sela())
                .selm()                         // MCLK SEL
                .variant(self.mclk.0.selm())
                .divm()                         // MCLK DIV
                .variant(self.mclk_div)
                .sels()                         // SMCLK SEL
                .variant(self.mclk.0.selm());
            match self.smclk.div() {            // SMCLK DIV
                Some(div) => w.divs().variant(div),
                None => w.divs().variant(0),
            }
        });
    }
}

impl ClockConfig<MclkDefined, SmclkDefined> {
    /// Apply clock configuration to hardware and return clock objects
    #[inline]
    pub fn freeze(self) -> Clocks {

        /// CSKEY
        const CS_STAT: u32 = 0b00011111;

        let mclk_freq = self.mclk.0.freq() >> (self.mclk_div as u32);
        let hsmclk = mclk_freq;

        cs_key(true);

        /* Waiting for the clock source ready bit to be valid before changing */
        wait_clk(CS_STAT);

        self.configure_dco_fll();
        self.configure_hfxt();
        self.configure_refo();
        self.configure_cs();

        let clocks = Clocks {
            aclk: Hertz(Aclk(self.aclk_sel.freq())),
            mclk: Hertz(mclk_freq),
            hsmclk: Hertz(hsmclk),
            smclk: Hertz(Smclk(mclk_freq >> (self.smclk.0 as u32))),
            bclk: Hertz(Aclk(self.aclk_sel.freq())),
        };

        /* Waiting for the clock source ready bit to be valid before changing */
        wait_clk(CS_STAT);

        cs_key(false);

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
    /// Returns the Auxiliary clock frequency
    pub fn aclk(&self) -> Hertz {
        self.aclk
    }

    /// Returns the Master clock frequency
    pub fn mclk(&self) -> Hertz {
        self.mclk
    }

    /// Returns the Subsystem master clock frequency
    pub fn hsmclk(&self) -> Hertz {
        self.hsmclk
    }

    /// Returns the Low-speed subsystem master clock frequency
    pub fn smclk(&self) -> Hertz {
        self.smclk
    }

    /// Returns the Low-speed backup domain clock frequency
    pub fn bclk(&self) -> Hertz {
        self.bclk
    }
}