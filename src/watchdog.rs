//! HAL library for WDT_A (Watchdog) Peripheral - MSP432P401R

use core::convert::Infallible;
use crate::common::*;
pub use hal::watchdog::{Disable, Enable, Watchdog};
use pac::WDT_A;

const WDT_COUNTER_CLEAR: u8 = 0x08;
const WDT_COUNTER_MASK: u8 = 0xF7;
const WDT_MODE_SELECT: u8 = 0x10;
const WDT_MODE_MASK: u8 = 0xEF;
const WDT_CONTROL_HOLD: u8 = 0x80;
const WDT_CONTROL_MASK: u8 = 0x7F;
const WDT_SOURCE_MASK: u8 = 0x9F;
const WDT_INTERVAL_MASK: u8 = 0xF8;

enum Mode {
    Timer,
    Watchdog,
}

#[derive(Copy, Clone)]
pub enum ClockSource {
    SMCLK = 0x0000,
    ACLK = 0x0020,
    VLOCLK = 0x0040,
    BCLK = 0x0060,
}

/// Timer interval at powers of 2
#[derive(Copy, Clone)]
pub enum TimerInterval {
    At31 = 0x0000,
    At27 = 0x0001,
    At23 = 0x0002,
    At19 = 0x0003,
    At15 = 0x0004,
    At13 = 0x0005,
    At9 = 0x0006,
    At6 = 0x0007,
}

pub struct Options(ClockSource, TimerInterval);

pub trait State {}

pub struct Enabled;

pub struct Disabled;

impl State for Enabled {}

impl State for Disabled {}

impl Constrain<WatchdogTimer<Enabled>> for WDT_A {
    fn constrain(self) -> WatchdogTimer<Enabled> {
        WatchdogTimer::<Enabled>::new(self)
    }
}

pub struct WatchdogTimer<S: State> {
    wdt: WDT_A,
    state: S,
}

impl<S> WatchdogTimer<S> where S: State {

    pub fn current_state(&self) -> &S {
        &self.state
    }

    fn stop_watchdog_timer(&self) {
        self.set_reg_mask(WDT_CONTROL_HOLD, WDT_CONTROL_MASK);
    }

    fn start_watchdog_timer(&self) {
        self.set_reg_mask(!WDT_CONTROL_HOLD, WDT_CONTROL_MASK);
    }

    pub fn set_timer_interval(&self, interval: TimerInterval) -> &Self {
        self.set_reg_mask(interval as u8,WDT_INTERVAL_MASK);
        self
    }

    pub fn set_clock_source(&self, source: ClockSource) {
        self.set_reg_mask(source as u8, WDT_SOURCE_MASK);
    }

    fn setup(&self, options: Options) {
        self.set_reg_mask(WDT_COUNTER_CLEAR, WDT_COUNTER_MASK);
        self.set_clock_source(options.0);
        self.set_timer_interval(options.1);
    }

    fn change_mode(&self, mode: Mode) {
        match mode {
            Mode::Timer => self.set_reg_mask(WDT_COUNTER_CLEAR | WDT_MODE_SELECT, WDT_COUNTER_MASK | WDT_MODE_MASK),
            Mode::Watchdog => self.set_reg_mask(WDT_COUNTER_CLEAR | !WDT_MODE_SELECT, WDT_COUNTER_MASK | WDT_MODE_MASK),
        }
    }

    fn set_reg_mask(&self, value: u8, mask: u8) {

        const WDT_PASSWORD: u16 = 0x5A00;

        self.wdt.wdtctl.modify(|r, w| unsafe {
            w.bits((r.bits() & mask as u16) | WDT_PASSWORD | value as u16)
        });
    }
}

impl WatchdogTimer<Enabled> {

    const fn new(wdt: WDT_A) -> WatchdogTimer<Enabled> {
        WatchdogTimer {
            wdt,
            state: Enabled,
        }
    }

    /// Set WDT to Watchdog mode
    pub fn select_watchdog_mode(&self) {
        self.change_mode(Mode::Watchdog)
    }

    /// Set WDT to Timer mode
    pub fn select_timer_mode(&self) {
        self.change_mode(Mode::Timer)
    }
}

impl Enable for WatchdogTimer<Disabled> {
    type Error = Infallible;
    type Time = Options;
    type Target = WatchdogTimer<Enabled>;

    fn try_start<T>(self, period: T) -> Result<Self::Target, Self::Error> where T: Into<Self::Time>,
    {
        self.setup(period.into());
        self.start_watchdog_timer();
        Ok(WatchdogTimer::<Enabled> { wdt: self.wdt, state: Enabled })
    }
}

impl Watchdog for WatchdogTimer<Enabled> {
    type Error = Infallible;

    fn try_feed(&mut self) -> Result<(), Self::Error> {
        self.set_reg_mask(WDT_COUNTER_CLEAR, WDT_COUNTER_MASK);
        Ok(())
    }
}

impl Disable for WatchdogTimer<Enabled> {
    type Error = Infallible;

    type Target = WatchdogTimer<Disabled>;

    fn try_disable(self) -> Result<Self::Target, Self::Error> {
        self.stop_watchdog_timer();
        Ok(WatchdogTimer::<Disabled> { wdt: self.wdt, state: Disabled })
    }
}
