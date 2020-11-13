// HAL library for WDT_A (Watchdog) Peripheral - MSP432P401R

/*****************************************************************************
    Config Example:

    mod watchdog;
    use pac::Peripherals;

    let peripherals = Peripherals::take().unwrap();                             // Take Peripheral access
    let watchdog = WatchdogTimer::<Enabled>::new(&peripherals.WDT_A);
    watchdog.try_disable().unwrap();                                            // Disable the watchdog

    let options = Options(ClockSource::BCLK, TimerInterval::At31);              // Config WDT time and clock source
    watchdog.try_start(options).unwrap();                                       // Enable the watchdog
    watchdog.try_feed();                                                        // Feed the watchdog counter

 ****************************************************************************/

use core::convert::Infallible;

pub use hal::watchdog::{Disable, Enable, Watchdog};
use pac::WDT_A;

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

struct WDT<'wdt> {
    wdt_a: &'wdt WDT_A,
}

impl<'wdt> WDT<'wdt> {
    const WDT_COUNTER_CLEAR: u16 = 0x0008;
    const WDT_MODE_SELECT: u16 = 0x0010;
    const WDT_CONTROL_HOLD: u16 = 0x0080;
    const WDT_PASSWORD: u16 = 0x5A00;
    const WDT_PASSWORD_MASK: u16 = 0x00FF;

    fn with_password(bits: u16) -> u16 {
        WDT::WDT_PASSWORD + (bits & WDT::WDT_PASSWORD_MASK)
    }

    pub fn set(&self, bits: u16) {
        self.wdt_a.wdtctl.modify(|r, w| unsafe {
            w.bits(WDT::with_password(r.bits() | bits))
        });
    }

    pub fn clear(&self, bits: u16) {
        self.wdt_a.wdtctl.modify(|r, w| unsafe {
            w.bits(WDT::with_password(r.bits() & bits))
        });
    }
}

pub struct WatchdogTimer<'wdt, S: State> {
    wdt: WDT<'wdt>,
    state: S,
}

impl<'wdt, S> WatchdogTimer<'wdt, S> where S: State {
    pub fn new(wdt_a: &WDT_A) -> WatchdogTimer<Enabled> {
        WatchdogTimer {
            wdt: WDT { wdt_a },
            state: Enabled,
        }
    }

    pub fn current_state(&self) -> &S {
        &self.state
    }

    fn stop_watchdog_timer(&self) {
        self.wdt.set(WDT::WDT_CONTROL_HOLD);
    }

    fn start_watchdog_timer(&self) {
        self.wdt.clear(!WDT::WDT_CONTROL_HOLD);
    }

    fn set_timer_interval(&self, interval: TimerInterval) {
        self.wdt.set(interval as u16);
    }

    fn set_clock_source(&self, source: ClockSource) {
        self.wdt.set(source as u16);
    }

    fn setup(&self, options: Options) {
        self.wdt.set(WDT::WDT_COUNTER_CLEAR);
        self.set_clock_source(options.0);
        self.set_timer_interval(options.1);
    }

    fn change_mode(&self, mode: Mode) {
        match mode {
            Mode::Timer => self.wdt.set(WDT::WDT_COUNTER_CLEAR | WDT::WDT_MODE_SELECT),
            Mode::Watchdog => {
                self.wdt.set(WDT::WDT_COUNTER_CLEAR);
                self.wdt.clear(!WDT::WDT_MODE_SELECT);
            }
        }
    }
}

impl<'wdt> WatchdogTimer<'wdt, Enabled> {
    /// Set WDT to Watchdog mode
    pub fn select_watchdog_mode(&self) {
        self.change_mode(Mode::Watchdog)
    }

    /// Set WDT to Timer mode
    pub fn select_timer_mode(&self) {
        self.change_mode(Mode::Timer)
    }
}

impl<'wdt> Enable for WatchdogTimer<'wdt, Disabled> {
    type Error = Infallible;
    type Time = Options;
    type Target = WatchdogTimer<'wdt, Enabled>;

    fn try_start<T>(self, period: T) -> Result<Self::Target, Self::Error> where T: Into<Self::Time>,
    {
        self.setup(period.into());
        self.start_watchdog_timer();
        Ok(WatchdogTimer::<Enabled> { wdt: self.wdt, state: Enabled })
    }
}

impl<'wdt> Watchdog for WatchdogTimer<'wdt, Enabled> {
    type Error = Infallible;

    fn try_feed(&mut self) -> Result<(), Self::Error> {
        self.wdt.set(WDT::WDT_COUNTER_CLEAR);
        Ok(())
    }
}

impl<'wdt> Disable for WatchdogTimer<'wdt, Enabled> {
    type Error = Infallible;

    type Target = WatchdogTimer<'wdt, Disabled>;

    fn try_disable(self) -> Result<Self::Target, Self::Error> {
        self.stop_watchdog_timer();
        Ok(WatchdogTimer::<Disabled> { wdt: self.wdt, state: Disabled })
    }
}
