// HAL library for WDT_A (Watchdog) Peripheral - MSP432P401R

/*****************************************************************************
    Config Example:

    mod watchdog;
    use pac::Peripherals;

    let peripherals = Peripherals::take().unwrap();                             // Take Peripheral access
    let wdta = peripherals.WDT_A;                                               // Take WDT_A
    let wdt: WatchdogTimer::<Enabled> = WatchdogTimer::<Enabled>::new(wdta);    // WDT_A always starts ON
    let wdt: WatchdogTimer::<Disabled> = wdt.try_disable().unwrap();            // Disable the watchdog

    let period: u16 = 0x0002;                                                   // Config WDT time and clock source (WDT_A_CTL_SSEL_0 | WDT_A_CTL_IS_2)
    let mut wdt: WatchdogTimer::<Enabled> = wdt.try_start(period).unwrap();     // Enable the watchdog
    wdt.try_feed();                                                             // Feed the watchdog counter

 ****************************************************************************/

use core::convert::Infallible;
use hal::watchdog::*;

use pac::WDT_A;

const WDT_A_CTL_IS_0: u16 = 0x0000;              /*< Watchdog clock source / (2^(31)) (18:12:16 at 32.768 kHz)  */
const WDT_A_CTL_IS_1: u16 = 0x0001;              /*< Watchdog clock source / (2^(27)) (01:08:16 at 32.768 kHz)  */
const WDT_A_CTL_IS_2: u16 = 0x0002;              /*< Watchdog clock source / (2^(23)) (00:04:16 at 32.768 kHz)  */
const WDT_A_CTL_IS_3: u16 = 0x0003;              /*< Watchdog clock source / (2^(19)) (00:00:16 at 32.768 kHz)  */
const WDT_A_CTL_IS_4: u16 = 0x0004;              /*< Watchdog clock source / (2^(15)) (1 s at 32.768 kHz)       */
const WDT_A_CTL_IS_5: u16 = 0x0005;              /*< Watchdog clock source / (2^(13)) (250 ms at 32.768 kHz)    */
const WDT_A_CTL_IS_6: u16 = 0x0006;              /*< Watchdog clock source / (2^(9))  (15.625 ms at 32.768 kHz) */
const WDT_A_CTL_IS_7: u16 = 0x0007;              /*< Watchdog clock source / (2^(6))  (1.95 ms at 32.768 kHz)   */

const WDT_COUNTER_CLEAR: u16 = 0x0008;

const WDT_MODE_SELECT: u16 = 0x0010;

const WDT_CLOCK_SOURCE_SMCLK: u16 = 0x0000;
const WDT_CLOCK_SOURCE_ACLK: u16 = 0x0020;
const WDT_CLOCK_SOURCE_VLOCLK: u16 = 0x0040;
const WDT_CLOCK_SOURCE_BCLK: u16 = 0x0060;

const WDT_CONTROL_HOLD: u16 = 0x0080;

const WDT_PASSWORD: u16 = 0x5A00;
const WDT_PASSWORD_MASK: u16 = 0x00FF;

pub struct Enabled;

pub struct Disabled;

enum Mode {
    Timer,
    Watchdog,
}

pub struct WatchdogTimer<T> {
    wdt_a: WDT_A,
    state: T,
}

impl<T> WatchdogTimer<T> {
    pub fn new(wdt_a: WDT_A) -> WatchdogTimer<Enabled> {
        WatchdogTimer { wdt_a, state: Enabled }
    }

    fn alter_control_bits<F: Fn(u16) -> u16>(&self, f: F) {
        self.wdt_a.wdtctl.modify(|r, w| unsafe {
            w.bits(WDT_PASSWORD + f(r.bits()) & WDT_PASSWORD_MASK)
        });
    }

    pub fn watchdog_read_state(&self) -> &T {
        &self.state
    }

    fn stop_watchdog_timer(&self) {
        self.alter_control_bits(|control: u16| {
            control | WDT_CONTROL_HOLD
        });
    }

    fn start_watchdog_timer(&self) {
        self.alter_control_bits(|control: u16| {
            (control | WDT_COUNTER_CLEAR) & !WDT_CONTROL_HOLD
        });
    }

    // Set the watchdog clock source and reset period
    // TODO: This is still very confusing
    fn period_watchdog_timer(&self, period: u16) {
        self.alter_control_bits(|control| {
            control
                | WDT_COUNTER_CLEAR
                & !(WDT_A_CTL_IS_7 | WDT_CLOCK_SOURCE_BCLK)
                | period & (WDT_A_CTL_IS_7 | WDT_CLOCK_SOURCE_BCLK)
        });
    }

    fn change_mode(&self, mode: Mode) {
        match mode {
            Mode::Timer => {
                self.alter_control_bits(|control| { control | WDT_COUNTER_CLEAR | WDT_MODE_SELECT });
            }
            Mode::Watchdog => {
                self.alter_control_bits(|control| { (control | WDT_COUNTER_CLEAR) & !WDT_MODE_SELECT });
            }
        }
    }
}

impl WatchdogTimer<Enabled> {
    // We may set another period when watchdog is enabled
    pub fn set_period(&self, period: u16) {
        self.period_watchdog_timer(period);
    }

    // Set WDT to Watchdog mode
    pub fn set_watchdog_mode(&self) {
        self.change_mode(Mode::Watchdog)
    }

    // Set WDT to Timer mode
    pub fn set_timer_mode(&self) {
        self.change_mode(Mode::Timer)
    }
}

impl Enable for WatchdogTimer<Disabled> {
    type Error = Infallible;
    type Time = u16;
    type Target = WatchdogTimer<Enabled>;

    fn try_start<T>(self, period: T) -> Result<WatchdogTimer<Enabled>, Self::Error> where T: Into<Self::Time>,
    {
        self.start_watchdog_timer();

        self.period_watchdog_timer(period.into());

        Ok(WatchdogTimer { wdt_a: self.wdt_a, state: Enabled })
    }
}

// We only implement `Watchdog` for a watchdog that is enabled.
// Application developers may not being able to `feed` an `Free<Disabled>`.
impl Watchdog for WatchdogTimer<Enabled> {
    type Error = Infallible;

    fn try_feed(&mut self) -> Result<(), Self::Error> {
        self.alter_control_bits(|control| { control | WDT_COUNTER_CLEAR });
        Ok(())
    }
}

impl Disable for WatchdogTimer<Enabled> {
    type Error = Infallible;

    type Target = WatchdogTimer<Disabled>;

    fn try_disable(self) -> Result<Self::Target, Self::Error> {
        self.stop_watchdog_timer();
        Ok(WatchdogTimer { wdt_a: self.wdt_a, state: Disabled })
    }
}
