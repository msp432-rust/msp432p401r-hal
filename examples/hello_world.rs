#![no_main]
#![no_std]
#![feature(llvm_asm)]

use cortex_m_rt::entry;
use panic_halt as _;
use cortex_m_semihosting::hprintln;

use msp432p401r_hal as hal;
use msp432p401r as pac;

use hal::watchdog::{WDTExt, Watchdog,TimerInterval};
use hal::gpio::{GpioExt, ToggleableOutputPin};
use hal::clock::{CsExt, MPrescaler, SMPrescaler, DCOFrequency};
use hal::pcm::{PcmExt, VCoreSel};
use hal::flash::{FlashExt, FlashWaitStates};

#[entry]
fn main() -> ! {

    // Take the Peripherals
    let p = pac::Peripherals::take().unwrap();

    // Watchdog Config.
    let mut _watchdog = p.WDT_A.constrain();                                 // Setup WatchdogTimer

    _watchdog.set_timer_interval(TimerInterval::At27);
    _watchdog.try_feed().unwrap();

    // PCM Config.
    let pcm = p.PCM.constrain()                                              // Setup PCM
        .set_vcore(VCoreSel::DcdcVcore1)                                     // Set DCDC Vcore1 -> 48 MHz Clock
        .freeze();
    let _pcm_sel = pcm.get_powermode();                                      // Get the current powermode

    // Flash Control Config.
    let _flctl = p.FLCTL.constrain()                                         // Setup Flash
        .set_waitstates(FlashWaitStates::_2Ws)                               // Two wait states -> 48 Mhz Clock
        .freeze();

    let _clock = p.CS.constrain()                                            // Setup CS
        .mclk_dcosource_selection(DCOFrequency::_48MHz, MPrescaler::DIVM_0)  // 48 MHz DCO
        .smclk_prescaler(SMPrescaler::DIVS_1)                                // 24 MHz SMCLK
        .freeze();

    hprintln!("Hello World Example - PCM: {}", _pcm_sel as u32).unwrap();

    let gpio = p.DIO.split();
    let mut p1_0 = gpio.p1_0.into_output();

    loop {
        _watchdog.try_feed().unwrap();
        p1_0.try_toggle().unwrap();
        let mut delay = 100000;
        while delay > 0 {
            delay = delay - 1;
        }
    }
}