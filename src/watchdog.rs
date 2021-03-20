//! HAL library for WDT_A (Watchdog) Peripheral - MSP432P401R

//! Usage example:
//! ```
//! #  use msp432p401r_hal::watchdog::{WatchdogTimer, Enabled, Disable, Enable};
//!
//! #  let watchdog = WatchdogTimer::<Enabled>::new();            // Setup WatchdogTimer
//! #  watchdog.try_disable().unwrap();                           // Disable the watchdog
//! ```

use core::convert::Infallible;
use core::marker::PhantomData;

pub use hal::watchdog::{Disable, Enable, Watchdog};
use pac::WDT_A;

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

pub struct Enabled {
    _marker: PhantomData<*const ()>,
}
pub struct Disabled {
    _marker: PhantomData<*const ()>,
}

pub struct WatchdogTimer<S> {
    wdt_a: WDT_A,
    _state: PhantomData<S>,
}

impl WatchdogTimer<()> {
    pub fn new(wdt_a: WDT_A) -> WatchdogTimer<Enabled> {
        WatchdogTimer { wdt_a, _state: PhantomData }
    }
}

impl<S> WatchdogTimer<S> {
    const WDT_COUNTER_CLEAR: u16 = 0x0008;
    const WDT_MODE_SELECT: u16 = 0x0010;
    const WDT_CONTROL_HOLD: u16 = 0x0080;
    const WDT_PASSWORD: u16 = 0x5A00;
    const WDT_PASSWORD_MASK: u16 = 0x00FF;

    #[inline(always)]
    const fn with_password(bits: u16) -> u16 {
        Self::WDT_PASSWORD + (bits & Self::WDT_PASSWORD_MASK)
    }

    fn set(&self, bits: u16) {
        self.wdt_a.wdtctl.modify(|r, w| unsafe {
            w.bits(Self::with_password(r.bits() | bits))
        });
    }

    fn clear(&self, bits: u16) {
        self.wdt_a.wdtctl.modify(|r, w| unsafe {
            w.bits(Self::with_password(r.bits() & !bits))
        });
    }
}

impl WatchdogTimer<Disabled> {
    #[inline(always)]
    fn start_watchdog_timer(&self) {
        self.clear(Self::WDT_CONTROL_HOLD);
    }

    #[inline(always)]
    fn set_timer_interval(&self, interval: TimerInterval) {
        self.set(interval as u16);
    }

    #[inline(always)]
    fn set_clock_source(&self, source: ClockSource) {
        self.set(source as u16);
    }

    #[inline(always)]
    fn setup(&self, options: Options) {
        self.set(Self::WDT_COUNTER_CLEAR);
        self.set_clock_source(options.0);
        self.set_timer_interval(options.1);
    }
}

impl WatchdogTimer<Enabled> {
    #[inline(always)]
    fn stop_watchdog_timer(&self) {
        self.set(Self::WDT_CONTROL_HOLD);
    }

    /// Set to watchdog mode.
    pub fn select_watchdog_mode(&self) {
        self.set(Self::WDT_COUNTER_CLEAR);
        self.clear(Self::WDT_MODE_SELECT);
    }

    /// Set to timer mode.
    #[inline(always)]
    pub fn select_timer_mode(&self) {
        self.set(Self::WDT_COUNTER_CLEAR | Self::WDT_MODE_SELECT);
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
        Ok(WatchdogTimer { wdt_a: self.wdt_a, _state: PhantomData })
    }
}

impl Watchdog for WatchdogTimer<Enabled> {
    type Error = Infallible;

    #[inline(always)]
    fn try_feed(&mut self) -> Result<(), Self::Error> {
        self.set(Self::WDT_COUNTER_CLEAR);
        Ok(())
    }
}

impl Disable for WatchdogTimer<Enabled> {
    type Error = Infallible;
    type Target = WatchdogTimer<Disabled>;

    fn try_disable(self) -> Result<Self::Target, Self::Error> {
        self.stop_watchdog_timer();
        Ok(WatchdogTimer { wdt_a: self.wdt_a, _state: PhantomData })
    }
}
