//! HAL library for PCM (Power Control Manager) - MSP432P401R

/*
- LDO regulator --> Low Dropout regulator, fast response;
- DC/DC regulator --> Higher efficiency and decreased current consumption from VCC;

-- POWER MODES: --

VCORE0 --> 24 MHz max. CPU frequency, 12 MHz max. input frequency;
VCORE1 --> 48 MHz max. CPU frequency, 24 MHz max. input frequency;

ACTIVE MODES (AM):

    - AM_LDO_VCORE0 --> Core voltage level 0 with LDO;
    - AM_LDO_VCORE1 --> Core voltage level 1 with LDO;
    - AM_DCDC_VCORE0 --> Core voltage level 0 with DC/DC;
    - AM_DCDC_VCORE1 --> Core voltage level 1 with DC/DC;
    - AM_LF_VCORE0 --> Core voltage level 0 with low frequency clock (max. 128 kHz);
    - AM_LF_VCORE1 --> Core voltage level 1 with low frequency clock (max. 128 kHz);

LOW POWER MODES:

LPM0 --> Sleep Mode, six modes of operation corresponding to each active mode;
LPM3 --> Deep Sleep, only LDO voltage options, max 32,768 kHz frequency;
LPM4 --> Deep Sleep, only LDO voltage options, with RTC and WDT modules disabled;
LPMx.5 --> Lowest power consumption

*/

use pac::pcm::pcmctl0::{AMR_A};
use pac::PCM;

/// Typestate for `PcmConfig` that represents unconfigured PCM
pub struct PcmNotDefined;
/// Typestate for `PcmConfig` that represents a configured PCM
pub struct PcmDefined;

pub trait State {}

impl State for PcmNotDefined {}

impl State for PcmDefined {}

#[derive(Copy, Clone, PartialEq)]
pub enum VCoreSel {
    LdoVcore0,
    LdoVcore1,
    DcdcVcore0,
    DcdcVcore1,
    LfVcore0,
    LfVcore1,
}

impl VCoreSel {
    #[inline(always)]
    fn vcoresel(&self) -> AMR_A {
        match *self {
            VCoreSel::LdoVcore0 => AMR_A::AMR_0,
            VCoreSel::LdoVcore1 => AMR_A::AMR_1,
            VCoreSel::DcdcVcore0 => AMR_A::AMR_4,
            VCoreSel::DcdcVcore1 => AMR_A::AMR_5,
            VCoreSel::LfVcore0 => AMR_A::AMR_8,
            VCoreSel::LfVcore1 => AMR_A::AMR_9,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum VCoreCheck {
    LdoVcore0,
    LdoVcore1,
    DcdcVcore0,
    DcdcVcore1,
    LfVcore0,
    LfVcore1,
    Lpm0LdoVcore0,
    Lpm0LdoVcore1,
    Lpm0DcdcVcore0,
    Lpm0DcdcVcore1,
    Lpm0LfVcore0,
    Lpm0LfVcore1,
    Lpm3,
}

/// Builder object that configures Power Control Manager (PCM)
pub struct PcmConfig <'a, S: State>{
    periph: &'a pac::pcm::RegisterBlock,
    _state: S,
    vcore_sel: VCoreSel,
}

impl <'a, S>PcmConfig<'a, S> where S: State{
    /// Converts PCM into a fresh, unconfigured PCM builder object
    pub fn new() -> PcmConfig<'a, PcmDefined> {

        let pcm = unsafe { &*PCM::ptr() };

        PcmConfig {
            periph: pcm,
            _state: PcmDefined,
            vcore_sel: VCoreSel::LdoVcore0,
        }
    }
}

impl <'a>PcmConfig<'a, PcmDefined> {

        #[inline]
        pub fn set_vcore(&mut self, source: VCoreSel) {

            let mut source_state: VCoreSel;
            source_state = self.get_vcore();

            source_state = match source_state {
                source if (source_state == source) => source,
                VCoreSel::DcdcVcore1 => VCoreSel::LdoVcore1,
                VCoreSel::LfVcore1 => VCoreSel::LdoVcore1,
                VCoreSel::DcdcVcore0 => VCoreSel::LdoVcore0,
                VCoreSel::LfVcore0 => VCoreSel::LdoVcore0,
                VCoreSel::LdoVcore1 if source == VCoreSel::LdoVcore0 ||
                source == VCoreSel::DcdcVcore0 || source == VCoreSel::LfVcore0
                => VCoreSel::LdoVcore0,
                VCoreSel::LdoVcore1 => source,
                VCoreSel::LdoVcore0 if source == VCoreSel::LdoVcore1 ||
                source == VCoreSel::DcdcVcore1 || source == VCoreSel::LfVcore1
                => VCoreSel::LdoVcore1,
                VCoreSel::LdoVcore0 => source,
            };

            self.set_vcore_inline(source_state);

            if source_state != source {
                self.set_vcore(source);
            }
        }

        #[inline]
        fn set_vcore_inline(&mut self, source: VCoreSel) {

            /// CSKEY
            const CSKEY: u16 = 0x695A;

            self.vcore_sel = source;

            loop {
                match self.periph.pcmctl1.read().pmr_busy().bits() as u8 {
                    0x00 => break,
                    _ => unsafe{llvm_asm!("NOP")},
                }
            };

            self.periph.pcmctl0.write(|w| unsafe {
                w.pcmkey().bits(CSKEY)
                 .amr().variant(self.vcore_sel.vcoresel())
            });

            loop {
                match self.periph.pcmctl1.read().pmr_busy().bits() as u8 {
                    0x00 => break,
                    _ => unsafe{llvm_asm!("NOP")},
                }
            };

             self.periph.pcmctl0.write(|w| unsafe {
                w.pcmkey().bits(!CSKEY)
             });
        }

        #[inline]
        fn get_vcore(&self) -> VCoreSel {
            match self.periph.pcmctl0.read().amr().bits() as u8 {
                0 => VCoreSel::LdoVcore0,
                1 => VCoreSel::LdoVcore1,
                4 => VCoreSel::DcdcVcore0,
                5 => VCoreSel::DcdcVcore1,
                8 => VCoreSel::LfVcore0,
                9 => VCoreSel::LfVcore1,
                _ => VCoreSel::LdoVcore0,
            }
        }

        #[inline]
        pub fn get_powermode() -> VCoreCheck {

        let pcm = unsafe { &*PCM::ptr() };

            match pcm.pcmctl0.read().cpm().bits() as u8 {
                0 => VCoreCheck::LdoVcore0,
                1 => VCoreCheck::LdoVcore1,
                4 => VCoreCheck::DcdcVcore0,
                5 => VCoreCheck::DcdcVcore1,
                8 => VCoreCheck::LfVcore0,
                9 => VCoreCheck::LfVcore1,
                16 => VCoreCheck::Lpm0LdoVcore0,
                17 => VCoreCheck::Lpm0LdoVcore1,
                20 => VCoreCheck::Lpm0DcdcVcore0,
                21 => VCoreCheck::Lpm0DcdcVcore1,
                24 => VCoreCheck::Lpm0LfVcore0,
                25 => VCoreCheck::Lpm0LfVcore1,
                32 => VCoreCheck::Lpm3,
                _ => VCoreCheck::LdoVcore0,
            }
        }
}


