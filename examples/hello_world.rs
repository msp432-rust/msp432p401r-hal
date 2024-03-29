#![no_main]
#![no_std]

use panic_halt as _;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nb::block;

use msp432p401r as pac;
use msp432p401r_hal as hal;

use hal::{clock::*, flash::*, gpio::*, pcm::*, timer::*, watchdog::*};

#[entry]
fn main() -> ! {

    // Take the Peripherals
    let p = pac::Peripherals::take().unwrap();

    // Watchdog Config.
    let mut _watchdog = p.WDT_A.constrain();                                 // Setup WatchdogTimer

    _watchdog.set_timer_interval(TimerInterval::At31);
    _watchdog.feed().unwrap();

    // PCM Config.
    let pcm = p.PCM.constrain()                                              // Setup PCM
        .set_vcore(VCoreSel::DcdcVcore1)                                     // Set DCDC Vcore1 -> 48 MHz Clock
        .freeze();
    let _pcm_sel = pcm.get_powermode();                                      // Get the current powermode

    // Flash Control Config.
    let _flash_control = p.FLCTL.constrain()                                         // Setup Flash
        .set_waitstates(FlashWaitStates::_2)                               // Two wait states -> 48 Mhz Clock
        .freeze();

    let _clock = p.CS.constrain()                                            // Setup CS
        .mclk_dcosource_selection(DCOFrequency::_48MHz, MPrescaler::DIVM_0)  // 48 MHz DCO
        .smclk_prescaler(SMPrescaler::DIVS_1)                                // 24 MHz SMCLK
        .freeze();

    hprintln!("Hello World Example - PCM: {}", _pcm_sel as u32);

    let gpio = p.DIO.split();
    let mut p1_0 = gpio.p1_0.into_output();

    let mut tim0 = p.TIMER_A0.constrain().set_clock(_clock);
    let count = Count(10, TimerUnit::Hertz);
    tim0.start(count).unwrap();

    loop {
        _watchdog.feed().unwrap();
        p1_0.toggle().unwrap();
        block!(tim0.wait()).unwrap();
    }
}
