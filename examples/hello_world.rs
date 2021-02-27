#![no_main]
#![no_std]
#![feature(llvm_asm)]

use cortex_m_rt::entry;
use panic_halt as _;
// use cortex_m_semihosting::hprintln;                                      // Enable debug print

extern crate msp432p401r_hal as hal;
extern crate msp432p401r as pac;

use hal::watchdog::{WatchdogTimer, Enabled, Disable};
use hal::gpio::{Output, ToggleableOutputPin, GPIO};
use hal::gpio::porta::P1_0;
use hal::clock::{ClockConfig, DIVM_A, DIVS_A, Clocks, DcoclkFreqSel};
use hal::pcm::{PcmConfig, PcmDefined, VCoreSel};
use hal::flctl::{FlctlConfig, FlcDefined, FlWaitSts};

#[entry]
fn main() -> ! {

   // Watchdog Config.
   let watchdog = WatchdogTimer::<Enabled>::new();                          // Setup WatchdogTimer
   watchdog.try_disable().unwrap();                                         // Disable the watchdog

   // PCM Config.
   let mut pcm: PcmConfig::<PcmDefined> = PcmConfig::<PcmDefined>::new();   // Setup PcmConfig
           pcm.set_vcore(VCoreSel::DcdcVcore1);                             // Set DCDC Vcore1 -> 48 MHz Clock
   let _pcm_sel = pcm.get_powermode();                                      // Get the current powermode

   // Flash Control Config.
   let flctl = FlctlConfig::<FlcDefined>::new();                            // Setup FlctlConfig
   flctl.set_flwaitst(FlWaitSts::_2Ws);                                     // Two wait states -> 48 Mhz Clock

   // hprintln!("Hello World Example").unwrap();

   let mut p1_0: P1_0<GPIO<Output>> = P1_0::<GPIO<Output>>::into_output();

   let _clock :Clocks = ClockConfig::new()
        .mclk_dcoclk( DcoclkFreqSel::_48MHz, DIVM_A::DIVM_0)
        .smclk_div(DIVS_A::DIVS_1)
        .freeze();

    loop {
        p1_0.try_toggle().unwrap();
        let mut delay = 100000;
        while delay > 0 {
            delay = delay - 1;
        }
    }
}