use crate::pac::{TIMER_A0, TIMER_A1, TIMER_A2, TIMER_A3};

use crate::clock::Clocks;
use crate::time::Hertz;

#[derive(Debug, Clone, Copy)]
pub enum ClockSource {
    EXTERNAL_TXCLK,
    ACLK,
    SMCLK,
    INVERTED_EXTERNAL_TXCLK,
}

#[derive(Debug, Clone, Copy)]
pub enum ClockSourceDivider {
     _1 =  1,
     _2 =  2,
     _4 =  4,
     _8 =  8,
     _3 =  3,
     _5 =  5,
     _6 =  6,
     _7 =  7,
    _10 = 10,
    _12 = 12,
    _14 = 14,
    _16 = 16,
    _20 = 20,
    _24 = 24,
    _28 = 28,
    _32 = 32,
    _40 = 40,
    _48 = 48,
    _56 = 56,
    _64 = 64,
}

#[derive(Debug)]
pub struct UpTimerConfig {
    pub clock_source: ClockSource,
    pub divider: ClockSourceDivider,
    pub period: u16,
    pub interrupt_enabled: bool,
    pub capture_compare_interrupt_enabled: bool,
    pub clear: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputMode {
    OutBitValue,
    Set,
    ToggleReset,
    SetReset,
    Toggle,
    Reset,
    ToggleSet,
    ResetSet,
}

#[derive(Debug)]
pub struct CompareConfig {
    pub capture_compare_interrupt_enabled: bool,
    pub output_mode: OutputMode,
    pub compare_value: u16,
}

pub struct Timer<TIMER> {
    pub timer: TIMER,
   // pub(crate) clock: Hertz,
}

pub struct UpTimer<TIMER> {
    pub timer: TIMER,
   // pub(crate) clock: Hertz,
}

macro_rules! hal {
    ($($TIMER_AX:ident: ($timer_ax:ident, $ax:ident),)+) => {
        $(
            impl Timer<()> {
                pub fn $ax(timer: $TIMER_AX, /*clocks: &Clocks*/) -> Timer<$TIMER_AX> {
                    Timer { timer,/* clock: clocks.smclk()*/ }
                }
            }

            impl UpTimer<$TIMER_AX> {
              pub fn start(&mut self) {
                self.timer.tax_ctl.modify(|_, w| w.mc().mc_1());
              }

              pub fn clear_capture_compare_interrupt(&mut self, register: usize) {
                self.timer.tax_cctl[register].modify(|_, w| w.ccifg().ccifg_0())
              }

              pub fn set_compare_config(&mut self, register: usize, config: &CompareConfig) {
                use crate::pac::$timer_ax::tax_cctl::OUTMOD_A::*;

                if register == 0 {
                  assert!(matches!(
                     config.output_mode,
                     OutputMode::Toggle | OutputMode::Set | OutputMode::OutBitValue | OutputMode::Reset
                   ))
                }

                let outmod = match config.output_mode {
                  OutputMode::OutBitValue => OUTMOD_0,
                  OutputMode::Set => OUTMOD_1,
                  OutputMode::ToggleReset => OUTMOD_2,
                  OutputMode::SetReset => OUTMOD_3,
                  OutputMode::Toggle => OUTMOD_4,
                  OutputMode::Reset => OUTMOD_5,
                  OutputMode::ToggleSet => OUTMOD_6,
                  OutputMode::ResetSet => OUTMOD_7,
                };

                self.timer.tax_cctl[register].modify(|_, w| w.ccie().bit(config.capture_compare_interrupt_enabled)
                  .outmod().variant(outmod)
                  .cap().clear_bit()
                );

                self.timer.tax_ccr[register].modify(|_, w| unsafe { w.tax_r().bits(config.compare_value) });

              }

              pub fn set_compare_value(&mut self, register: usize, compare_value: u16) {
                self.timer.tax_ccr[register].modify(|_, w| unsafe { w.tax_r().bits(compare_value) })
              }
            }

            impl Timer<$TIMER_AX> {
                pub fn start_count_down<T>(self, timeout: impl Into<Hertz>) {
                    let Self { timer, /*clock*/ } = self;

                }

                pub fn up(mut self, config: &UpTimerConfig) -> UpTimer<$TIMER_AX> {
                  use crate::pac::$timer_ax::tax_ctl::TASSEL_A::*;

                  self.set_divider(config.divider);

                  self.timer.tax_ctl.modify(|_, w| w.tassel().variant(match config.clock_source {
                      ClockSource::EXTERNAL_TXCLK          => TASSEL_0,
                      ClockSource::ACLK                    => TASSEL_1,
                      ClockSource::SMCLK                   => TASSEL_2,
                      ClockSource::INVERTED_EXTERNAL_TXCLK => TASSEL_3,
                    })
                     .mc().mc_0()
                     .taclr().bit(config.clear)
                     .taie().bit(config.interrupt_enabled)
                  );

                  self.timer.tax_cctl[0].modify(|_, w| w.ccie().bit(config.capture_compare_interrupt_enabled));

                  self.timer.tax_ccr[0].modify(|_, w| unsafe { w.tax_r().bits(config.period) });

                  UpTimer { timer: self.timer }
                }

                fn set_divider(&mut self, divider: ClockSourceDivider) {
                    use ClockSourceDivider::*;

                    match divider {
                        _1 | _2 => {
                            self.timer.tax_ctl.modify(|_, w| unsafe { w.id().bits(divider as u8 - 1) });
                            self.timer.tax_ex0.modify(|_, w| w.taidex().taidex_0());
                        },
                        _4 => {
                            self.timer.tax_ctl.modify(|_, w| w.id().id_2());
                            self.timer.tax_ex0.modify(|_, w| w.taidex().taidex_0());
                        }
                        _8 => {
                            self.timer.tax_ctl.modify(|_, w| w.id().id_3());
                            self.timer.tax_ex0.modify(|_, w| w.taidex().taidex_0());
                        },
                        _3 | _5 | _6 | _7 => {
                            self.timer.tax_ctl.modify(|_, w| w.id().id_0());
                            self.timer.tax_ex0.modify(|_, w| unsafe { w.bits(divider as u16 - 1) });
                        },
                        _10 | _12 | _14 | _16 => {
                            self.timer.tax_ctl.modify(|_, w| w.id().id_1());
                            self.timer.tax_ex0.modify(|_, w| unsafe { w.bits(divider as u16 / 2 - 1) });
                        },
                        _20 | _24 | _28 | _32 => {
                            self.timer.tax_ctl.modify(|_, w| w.id().id_2());
                            self.timer.tax_ex0.modify(|_, w| unsafe { w.bits(divider as u16 / 4 - 1) });
                        },
                        _40 | _48 | _56 | _64 => {
                            self.timer.tax_ctl.modify(|_, w| w.id().id_3());
                            self.timer.tax_ex0.modify(|_, w| unsafe { w.bits(divider as u16 / 8 - 1) });
                        },
                    }
                }
            }
        )+
    }
}

hal! {
    TIMER_A0: (timer_a0, a0),
    TIMER_A1: (timer_a1, a1),
    TIMER_A2: (timer_a2, a2),
    TIMER_A3: (timer_a3, a3),
}
