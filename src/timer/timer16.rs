//! HAL library for Timer module (TimerA) - MSP432P401R
pub use embedded_hal::timer::Periodic;
pub use hal::timer::nb::{Cancel, CountDown};

use pac::TIMER_A0;
use pac::TIMER_A1;
use pac::TIMER_A2;
use pac::TIMER_A3;

use crate::clock::Clocks;
use crate::time::Hertz;
use crate::timer::*;

use ClockSource::*;
use ClockSourcePrescaler::*;

const MAX_PRESCALER: u32 = 0x0040;
const MAX_COUNT: u32 = 0xFFFF;

#[derive(Debug, Clone, Copy)]
pub enum ClockSource {
    ExternalTxclk,
    Aclk,
    Smclk,
    InvertedExternalTxclk,
}

#[derive(Debug, Clone, Copy)]
enum ClockSourcePrescaler {
    _1 =  1,
    _2 =  2,
    _3 =  3,
    _4 =  4,
    _5 =  5,
    _6 =  6,
    _7 =  7,
    _8 =  8,
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

pub struct TimerConfig<T, S: State> {
    clocks: Clocks,
    tim: T,
    _state: S,
    count: u16,
}

pub trait TimerExt {
    type Output;
    fn constrain(self) -> Self::Output;
}

macro_rules! timer {
    ($($TIMER:ident, $tim:ident),*) => {
        $(
            impl TimerExt for $TIMER {

                type Output = TimerConfig<$TIMER, ClockNotDefined>;

                fn constrain(self) -> Self::Output {
                    TimerConfig::<$TIMER, ClockNotDefined>::$tim(self)
                }
            }

            impl TimerConfig <$TIMER, ClockNotDefined> {

                fn $tim(timer: $TIMER) -> TimerConfig <$TIMER, ClockNotDefined>{

                    let hz: Hertz = Hertz(0);
                    let clock: Clocks = Clocks{aclk: hz, mclk: hz, hsmclk: hz, smclk: hz, bclk: hz };

                    TimerConfig {
                        clocks: clock,
                        tim: timer,
                        _state: ClockNotDefined,
                        count: 0,
                    }
                }

                pub fn set_clock(self, clock: Clocks) -> TimerConfig <$TIMER, ClockDefined> {
                    TimerConfig {
                        clocks: clock,
                        tim: self.tim,
                        _state: ClockDefined,
                        count: 0,
                    }
                }
            }

            impl TimerConfig <$TIMER, ClockDefined> {

                #[inline]
                pub fn update_clock(&mut self, clocks: Clocks) -> &mut Self {
                    self.clocks = clocks;
                    self
                }

                #[inline]
                pub fn enable_interrupt(&mut self) -> &mut Self{
                    self.tim.tax_ctl.modify(|_, w| w.taie().taie_1());
                    self
                }

                #[inline]
                pub fn disable_interrupt(&mut self) -> &mut Self {
                    self.tim.tax_ctl.modify(|_, w| w.taie().taie_0());
                    self
                }

                #[inline]
                pub fn interrupt_enabled(&self) -> bool {
                    self.tim.tax_ctl.read().taie().is_taie_1() == true
                }

                #[inline]
                pub fn clear_interrupt_pending_bit(&mut self) -> &mut Self {
                    self.tim.tax_ctl.modify(|_, w| w.taifg().clear_bit());
                    self
                }

                #[inline]
                pub fn check_interrupt(&self) -> bool {
                    self.tim.tax_ctl.read().taifg().is_taifg_1() == true
                }

                #[inline]
                fn stop_timer(&mut self) {
                    self.tim.tax_ctl.modify(|_, w| w.mc().mc_0());
                }

                #[inline]
                fn timer_wrapped(&mut self) -> bool {
                    let val: u16 = self.tim.tax_r.read().bits();
                        if(self.count > val && self.count >= self.tim.tax_ccr[0].read().bits() - 1) {
                            self.count = 0;
                            true
                        } else {
                            self.count = val;
                            false
                        }
                }

                #[inline]
                fn timer_running(&self) -> bool {
                    self.tim.tax_ctl.read().mc().is_mc_0() == false
                }

                #[inline]
                fn clear_timer(&mut self) {
                    self.tim.tax_ctl.modify(|_, w| w.taclr().set_bit());
                    self.count = 0;
                }

                #[inline]
                fn setup_timer(&self, count: Count) -> bool {
                    use TimerUnit::*;
                    match count.1 {
                        Hertz | Milliseconds => self.setup_count(count),
                        Kilohertz => self.setup_count(Count(count.0 * 1000, Hertz)),
                        Seconds => self.setup_count(Count(count.0 * 1000, Milliseconds)),
                    }
                }

                #[inline]
                fn setup_count(&self, count: Count) -> bool {

                    let max_period = MAX_PRESCALER * MAX_COUNT;
                    let aclk_ratio: u32;
                    let smclk_ratio : u32;
                    let mut count_ratio : u32  = 0;
                    let mut clock_source : ClockSource = Smclk;

                    if count.0.checked_mul(self.clocks.aclk.0) == None {
                        return false;
                    }

                    if count.1 == TimerUnit::Hertz {
                        let frequency = count.0;
                        aclk_ratio = self.clocks.aclk.0 / frequency;
                        smclk_ratio = self.clocks.smclk.0 / frequency;
                    } else {
                        let period = count.0;
                        aclk_ratio = (period*self.clocks.aclk.0)/1000;
                        smclk_ratio = period*(self.clocks.smclk.0/1000);
                    }

                    if(smclk_ratio < max_period) {
                        count_ratio = smclk_ratio;
                        clock_source = Smclk;
                    } else if (aclk_ratio < max_period) {
                        count_ratio = aclk_ratio;
                        clock_source = Aclk;
                    }

                    if count_ratio != 0 {
                        let mut min_prescaler = (count_ratio / MAX_COUNT) as u8;

                        // In period, prescaler need to be above the min_prescaler value
                        if count.1 != TimerUnit::Hertz {
                            min_prescaler = min_prescaler + 1;
                        }

                        let real_prescaler = self.get_prescaler(min_prescaler);
                        let tick_count = (count_ratio / real_prescaler as u32) as u16;
                        self.set_clock_source(clock_source);
                        self.set_count(tick_count);
                        true
                    } else {
                        false
                    }
                }

                fn set_clock_source(&self, source: ClockSource) {
                  self.tim.tax_ctl.modify(|_, w| match source {
                      ExternalTxclk         => w.tassel().tassel_0(),
                      Aclk                  => w.tassel().tassel_1(),
                      Smclk                 => w.tassel().tassel_2(),
                      InvertedExternalTxclk => w.tassel().tassel_3(),
                    });
                }

                #[inline]
                fn set_count(&self, count: u16) {
                    self.tim.tax_ccr[0].modify(|r, w| unsafe {
                        w.bits(r.bits() | count)
                    });
                }

                #[inline]
                fn get_prescaler(&self, min_prescaler: u8) -> ClockSourcePrescaler {

                    match min_prescaler {
                        0..=1 => self.setup_prescaler(_1),
                        2 => self.setup_prescaler(_2),
                        3 => self.setup_prescaler(_3),
                        4 => self.setup_prescaler(_4),
                        5 => self.setup_prescaler(_5),
                        6 => self.setup_prescaler(_6),
                        7 => self.setup_prescaler(_7),
                        8 => self.setup_prescaler(_8),
                        9..=10 => self.setup_prescaler(_10),
                        11..=12 => self.setup_prescaler(_12),
                        13..=14 => self.setup_prescaler(_14),
                        15..=16 => self.setup_prescaler(_16),
                        17..=20 => self.setup_prescaler(_20),
                        21..=24 => self.setup_prescaler(_24),
                        25..=28 => self.setup_prescaler(_28),
                        29..=32 => self.setup_prescaler(_32),
                        33..=40 => self.setup_prescaler(_40),
                        41..=48 => self.setup_prescaler(_48),
                        49..=56 => self.setup_prescaler(_56),
                        _ => self.setup_prescaler(_64),
                    }
                }

                #[inline]
                fn setup_prescaler(&self, prescaler: ClockSourcePrescaler) -> ClockSourcePrescaler {

                    match prescaler {
                        _1 | _2 | _3 | _4 | _5 | _6 | _7 | _8 => {
                            self.tim.tax_ctl.modify(|_,w| w.id().id_0());
                            self.tim.tax_ex0.modify(|_, w| unsafe {w.bits(prescaler as u16 -1)} );
                        },
                        _10 | _12 | _14 | _16 => {
                            self.tim.tax_ctl.modify(|_,w| w.id().id_1());
                            self.tim.tax_ex0.modify(|_, w| unsafe {w.bits(prescaler as u16 / 2 -1)} );
                        },
                        _20 | _24 | _28 | _32 => {
                            self.tim.tax_ctl.modify(|_,w| w.id().id_2());
                            self.tim.tax_ex0.modify(|_, w| unsafe {w.bits(prescaler as u16 / 4 -1)} );
                        },
                        _40 | _48 | _56 | _64 => {
                            self.tim.tax_ctl.modify(|_,w| w.id().id_3());
                            self.tim.tax_ex0.modify(|_, w| unsafe {w.bits(prescaler as u16 / 8 -1)} );
                        },
                    }

                    prescaler
                }

                #[inline]
                fn start_timer(&self) {
                    self.tim.tax_ctl.modify(|_, w| w.mc().mc_1());
                }
            }

            impl CountDown for TimerConfig <$TIMER, ClockDefined> {
                type Error = Error;
                type Time = Count;

                fn try_start <T>(&mut self, count: T) -> Result<(), Self::Error>
                where
                    T: Into<Self::Time> {
                    if(!self.timer_running()) {
                        self.clear_timer();

                        if(self.setup_timer(count.into())) {
                            self.start_timer();
                            Ok(())
                        } else {
                            Err(Error::Unreachable)
                        }

                    } else {
                        Err(Error::Enabled)
                    }
                }

                fn try_wait(&mut self) -> nb::Result<(), Self::Error> {
                   if(self.timer_wrapped()) {
                        Ok(())
                   } else {
                        Err(nb::Error::WouldBlock)
                   }
                }
            }

            impl Cancel for TimerConfig <$TIMER, ClockDefined> {
                fn try_cancel(&mut self) -> Result<(), Self::Error> {
                    if(self.timer_running()) {
                        self.stop_timer();
                        Ok(())
                    } else {
                        Err(Error::Disabled)
                    }
                }
            }

            impl Periodic for TimerConfig <$TIMER, ClockDefined> {}
        )*
    };
}

timer! {
    TIMER_A0, tim0,
    TIMER_A1, tim1,
    TIMER_A2, tim2,
    TIMER_A3, tim3
}