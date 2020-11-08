//! # Watchdog (WDT_A)
//!
//! HAL Lib for WDT_A (Watchdog) Peripheral - MSP432P401R

/******************************************************************************
* WDT_A Bits
******************************************************************************/
/* WDT_A_CTL[IS] Bits */
//const WDT_A_CTL_IS_0:               u16 = 0x0000;              /*< Watchdog clock source / (2^(31)) (18:12:16 at 32.768 kHz) */
//const WDT_A_CTL_IS_1:               u16 = 0x0001;              /*< Watchdog clock source /(2^(27)) (01:08:16 at 32.768 kHz) */
//const WDT_A_CTL_IS_2:               u16 = 0x0002;              /*< Watchdog clock source /(2^(23)) (00:04:16 at 32.768 kHz) */
//const WDT_A_CTL_IS_3:               u16 = 0x0003;              /*< Watchdog clock source /(2^(19)) (00:00:16 at 32.768 kHz) */
//const WDT_A_CTL_IS_4:               u16 = 0x0004;              /*< Watchdog clock source /(2^(15)) (1 s at 32.768 kHz) */
//const WDT_A_CTL_IS_5:               u16 = 0x0005;              /*< Watchdog clock source / (2^(13)) (250 ms at 32.768 kHz) */
//const WDT_A_CTL_IS_6:               u16 = 0x0006;              /*< Watchdog clock source / (2^(9)) (15.625 ms at 32.768 kHz) */
const WDT_A_CTL_IS_7:               u16 = 0x0007;              /*< Watchdog clock source / (2^(6)) (1.95 ms at 32.768 kHz) */
/* WDT_A_CTL[CNTCL] Bits */
const WDT_A_CTL_CNTCL:              u16 = 0x0008;              /*< Watchdog timer counter clear */
/* WDT_A_CTL[TMSEL] Bits */
const WDT_A_CTL_TMSEL:              u16 = 0x0010;              /*< Watchdog timer mode select */
/* WDT_A_CTL[SSEL] Bits */
//const WDT_A_CTL_SSEL_0:             u16 = 0x0000;              /*< SMCLK */
//const WDT_A_CTL_SSEL_1:             u16 = 0x0020;              /*< ACLK */
//const WDT_A_CTL_SSEL_2:             u16 = 0x0040;              /*< VLOCLK */
const WDT_A_CTL_SSEL_3:             u16 = 0x0060;              /*< BCLK */
/* WDT_A_CTL[HOLD] Bits */
const WDT_A_CTL_HOLD:               u16 = 0x0080;              /*< Watchdog timer hold */
/* WDT_A_CTL[PW] Bits */
const WDT_A_CTL_PW_MASK:            u16 = 0x00FF;              /*< WDTPW Bit Mask */
/* Pre-defined bitfield values */
const WDT_A_CTL_PW:                 u16 = 0x5A00;              /*< WDT Key Value for WDT write access */
/******************************************************************************/

/*****************************************************************************
    Config Example:

    pub mod watchdog;

    let peripherals = Peripherals::take().unwrap();                             // Take Peripheral access
    let wdta = peripherals.WDT_A;                                               // Take WDT_A
    let wdt: WatchdogTimer::<Enabled> = WatchdogTimer::<Enabled>::new(wdta);    // WDT_A always starts ON
    let wdt: WatchdogTimer::<Disabled> = wdt.try_disable().unwrap();            // Disable the watchdog

    let period: u16 = 0x0002;                                                   // Config WDT time and clock source (WDT_A_CTL_SSEL_0 | WDT_A_CTL_IS_2)
    let mut wdt: WatchdogTimer::<Enabled> = wdt.try_start(period).unwrap();     // Enable the watchdog
    let _ = wdt.try_feed();                                                     // Feed the watchdog counter

 ****************************************************************************/

pub use crate::{
    hal::watchdog::{Watchdog, Enable, Disable},
    pac::WDT_A,
};

use core::convert::Infallible;

pub struct Disabled;                    // Struct for disabled WDT
pub struct Enabled;                     // Struct for enabled WDT

/// Wraps the Watchdog Timer (WDT_A) peripheral
pub struct WatchdogTimer<STATE> {
    wdt_a: WDT_A,
    state: STATE,
}

impl<STATE> WatchdogTimer<STATE> {

    /// WDT_A Pub Funcs
    // Wrap the watchdog
    pub fn new(wdt_a: WDT_A) -> WatchdogTimer<Enabled> {
        WatchdogTimer { wdt_a, state: Enabled }
    }

    // Read WDT State
    pub fn watchdog_read_state(&self) -> &STATE {
        &self.state
    }

    // Stop the watchdog
    // Set WDT_A_CTL_HOLD bit - Bit 7
    fn stop_watchdog_timer(&self) {
        self.wdt_a.wdtctl.modify(|r, w| unsafe {
            w.bits(WDT_A_CTL_PW + ((r.bits() | WDT_A_CTL_HOLD) & WDT_A_CTL_PW_MASK))
        });
    }

    // Start the watchdog
    // Reset WDT_A_CTL_HOLD bit - Bit 7
    fn start_watchdog_timer(&self) {
        self.wdt_a.wdtctl.modify(|r, w| unsafe {
            w.bits(WDT_A_CTL_PW + ( ( (r.bits() | WDT_A_CTL_CNTCL) & !WDT_A_CTL_HOLD ) & WDT_A_CTL_PW_MASK ) )
        });
    }

    // Set the watchdog clock source and reset period
    fn period_watchdog_timer(&self, period: u16) {
        self.wdt_a.wdtctl.modify(|r, w| unsafe {
            let val = period & (WDT_A_CTL_IS_7 | WDT_A_CTL_SSEL_3);        // Mask Safe values
            w.bits(WDT_A_CTL_PW + ( ( ( (r.bits() | WDT_A_CTL_CNTCL) & !(WDT_A_CTL_IS_7 | WDT_A_CTL_SSEL_3) ) | val) & WDT_A_CTL_PW_MASK) )
        });
    }

    // Set the the Watchdog timer mode
    fn mode_watchdog_timer(&self, mode:bool) {

        if mode == true {
            self.wdt_a.wdtctl.modify(|r, w| unsafe {
                w.bits(WDT_A_CTL_PW + ( ( (r.bits() | WDT_A_CTL_CNTCL) | WDT_A_CTL_TMSEL ) & WDT_A_CTL_PW_MASK ) )
            });
        } else {
            self.wdt_a.wdtctl.modify(|r, w| unsafe {
                w.bits(WDT_A_CTL_PW + ( ( (r.bits() | WDT_A_CTL_CNTCL) & !WDT_A_CTL_TMSEL ) & WDT_A_CTL_PW_MASK ) )
            });
        }
    }
}

impl WatchdogTimer<Enabled> {
    // We may set another period when watchdog is enabled
    pub fn set_period(&self, period: u16) {
        self.period_watchdog_timer(period);
    }

    // We may select the Watchdog timer mode
    pub fn set_mode(&self, mode: bool) {
        self.mode_watchdog_timer(mode);
    }
}

impl Enable for WatchdogTimer<Disabled> {
    type Error = Infallible;
    type Time = u16;
    type Target = WatchdogTimer<Enabled>;

    fn try_start<T>(self, period: T) -> Result<WatchdogTimer<Enabled>, Self::Error>
    where
        T: Into<Self::Time>,
    {

        // Start the watchdog
        self.start_watchdog_timer();

        //  Set period of the watchdog
        self.period_watchdog_timer(period.into());

        // Change typestate to `Enabled`
        Ok(WatchdogTimer { wdt_a: self.wdt_a, state: Enabled })
    }
}

// We only implement `Watchdog` for a watchdog that is enabled.
// Application developers may not being able to `feed` an `Free<Disabled>`.
impl Watchdog for WatchdogTimer<Enabled> {

    type Error = Infallible;

    fn try_feed(&mut self) -> Result<(), Self::Error> {

        // Reset the Watchdog counter
        // Set WDT_A_CTL_CNTCL bit - Bit 7
        self.wdt_a.wdtctl.modify(|r, w| unsafe {
            w.bits(WDT_A_CTL_PW + ((r.bits() | WDT_A_CTL_CNTCL) & WDT_A_CTL_PW_MASK))
        });

        Ok(())
    }
}

impl Disable for WatchdogTimer<Enabled> {

    /// For infallible implementations, will be `Infallible`
     type Error = Infallible;

     /// Disabled watchdog instance that can be enabled.
     type Target = WatchdogTimer<Disabled>;

     fn try_disable(self) -> Result<Self::Target, Self::Error> {

         // Stop the watchdog
         self.stop_watchdog_timer();

         // Change typestate to `Disabled`
         Ok(WatchdogTimer{ wdt_a: self.wdt_a, state: Disabled })
     }
}
