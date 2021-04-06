//! HAL library for Timer module (Timer32 and TimerA) - MSP432P401R
pub use embedded_hal::timer::{Cancel, CountDown, Periodic};
// TIMER 32 TO_DO!
//use pac::TIMER32;
use pac::TIMER_A0;
use pac::TIMER_A1;
use pac::TIMER_A2;
use pac::TIMER_A3;

use crate::clock::Clocks;
use crate::time::Hertz;

pub trait State {}

pub struct ClockNotDefined;

pub struct ClockDefined;

const MAX_PRESCALER: u32 = 0x0040;
const MAX_COUNT: u32 = 0xFFFF;

#[derive(Debug, Copy, Clone)]
pub enum TimerUnit {
    Hertz,
    Kilohertz,
    Milliseconds,
    Seconds,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Disabled,
    Enabled,
    Unreachable,
}

pub struct Count(pub u32, pub TimerUnit);

impl State for ClockNotDefined {}

impl State for ClockDefined {}

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
                pub fn update_clock(mut self, clocks: Clocks) -> Self {
                    self.clocks = clocks;
                    self
                }

                #[inline]
                pub fn enable_interrupt(&mut self) {
                    self.set_ctl(0x01, 0x01);
                }

                #[inline]
                pub fn disable_interrupt(&mut self) {
                    self.clear_ctl(0x01,0x01);
                }

                #[inline]
                pub fn interrupt_enabled(&self) -> bool {
                    self.tim.tax_ctl.read().bits() & (0x01 << 1) != 0
                }

                #[inline]
                pub fn clear_interrupt_pending_bit(&mut self) {
                    self.clear_ctl(0x01,0x00);
                }

                #[inline]
                pub fn check_interrupt(&self) -> bool {
                    self.tim.tax_ctl.read().bits() & (0x01 << 0) != 0
                }

                #[inline]
                fn stop_timer(&mut self) {
                    self.clear_ctl(0x03,0x04);
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
                    self.tim.tax_ctl.read().bits() & (0x03 << 4) != 0
                }

                #[inline]
                fn clear_timer(&mut self) {
                    self.set_ctl(0x01, 0x02);
                    self.count = 0;
                }

                #[inline]
                fn setup_timer(&self, count: Count) -> bool {
                    match count.1 {
                        TimerUnit::Hertz => self.setup_frequency(count.0),
                        TimerUnit::Kilohertz => self.setup_frequency(1000*count.0),
                        TimerUnit::Milliseconds => self.setup_period(count.0),
                        TimerUnit::Seconds => self.setup_period(1000*count.0),
                    }
                }

                fn set_clock_source(&self, value: u8, prescaler: u8) {
                    self.clear_ctl(0x0F, 0x06);
                    self.set_ctl((value | prescaler) as u16, 0x06);
                }

                fn setup_period(&self, period: u32) -> bool {
                    let smclk_period = period*(self.clocks.smclk.0/1000);
                    let aclk_period = (period*self.clocks.aclk.0)/1000;
                    let max_period = MAX_PRESCALER * MAX_COUNT;

                    if smclk_period < max_period {
                        let prescaler = self.get_prescaler((smclk_period / MAX_COUNT) as u8 + 1);
                        let count = ((period * self.clocks.smclk.0) / (prescaler[2] as u32 * 1000)) as u16;
                        self.set_clock_source(0x08, prescaler[1]);
                        self.setup_count(count);
                        true
                    } else if aclk_period < max_period {
                        let prescaler = self.get_prescaler((aclk_period / MAX_COUNT) as u8 + 1);
                        let count = ((period * self.clocks.aclk.0) / (prescaler[2] as u32 * 1000)) as u16;
                        self.set_clock_source(0x04, prescaler[1]);
                        self.setup_count(count);
                        true
                    } else {
                        false
                    }
                }

                fn setup_frequency(&self, frequency: u32) -> bool {
                    let max_period = MAX_PRESCALER * MAX_COUNT;

                    if (frequency > (self.clocks.smclk.0 / max_period)) {
                        let prescaler = self.get_prescaler((self.clocks.smclk.0/(MAX_COUNT * frequency)) as u8);
                        let count = (self.clocks.smclk.0 / (frequency * prescaler[2] as u32)) as u16;
                        self.set_clock_source(0x08, prescaler[1]);
                        self.setup_count(count);
                        true
                    } else if (frequency > (self.clocks.aclk.0 / max_period)) {
                        let prescaler = self.get_prescaler((self.clocks.aclk.0/(MAX_COUNT * frequency)) as u8);
                        let count = (self.clocks.aclk.0 / (frequency * prescaler[2] as u32)) as u16;
                        self.set_clock_source(0x04, prescaler[1]);
                        self.setup_count(count);
                        true
                    } else {
                        false
                    }
                }

                #[inline]
                fn setup_count(&self, count: u16) {
                    self.tim.tax_ccr[0].modify(|r, w| unsafe {
                        w.bits(r.bits() | count)
                    });
                }

                #[inline]
                fn get_prescaler(&self, val: u8) -> [u8;3] {
                    match val {
                        0..=1 => self.setup_prescaler(0, 0),
                        2 => self.setup_prescaler(1, 0),
                        3 => self.setup_prescaler(2, 0),
                        4 => self.setup_prescaler(3, 0),
                        5 => self.setup_prescaler(4, 0),
                        6 => self.setup_prescaler(5, 0),
                        7 => self.setup_prescaler(6, 0),
                        8 => self.setup_prescaler(7, 0),
                        9..=10 => self.setup_prescaler(4, 1),
                        11..=12 => self.setup_prescaler(5, 1),
                        13..=14 => self.setup_prescaler(6, 1),
                        15..=16 => self.setup_prescaler(7, 1),
                        17..=20 => self.setup_prescaler(4, 2),
                        21..=24 => self.setup_prescaler(5, 2),
                        25..=28 => self.setup_prescaler(6, 2),
                        29..=32 => self.setup_prescaler(7, 2),
                        33..=40 => self.setup_prescaler(4, 3),
                        41..=48 => self.setup_prescaler(5, 3),
                        49..=56 => self.setup_prescaler(6, 3),
                        _ => self.setup_prescaler(7, 3),
                    }
                }

                #[inline]
                fn setup_prescaler(&self, divider: u8, val2: u8) -> [u8;3] {
                    self.tim.tax_ex0.modify(|r, w| unsafe {
                        w.bits(r.bits() | (divider as u16))
                    });
                    [divider, val2, 2u8.pow(val2.into())*(divider+1)]
                }

                #[inline]
                fn start_timer(&self) {
                    self.set_ctl(0x01, 0x04);
                }

                #[inline]
                fn set_ctl(&self, val: u16, shift: u8) {
                    self.tim.tax_ctl.modify(|r, w| unsafe {
                        w.bits(r.bits() | (val << shift))
                    });
                }

                #[inline]
                fn clear_ctl(&self, val: u16, shift: u8) {
                    self.tim.tax_ctl.modify(|r, w| unsafe {
                        w.bits(r.bits() & !(val << shift))
                    });
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
