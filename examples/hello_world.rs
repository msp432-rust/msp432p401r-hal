#![no_main]
#![no_std]
#![feature(llvm_asm)]

use panic_halt as _;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nb::block;

use msp432p401r as pac;
use msp432p401r_hal as hal;

use hal::common::*;
use hal::clock::{DCOFrequency, MPrescaler, SMPrescaler};
use hal::flash::{FlashWaitStates};
use hal::gpio::{GpioExt, ToggleableOutputPin};
use hal::pcm::CoreVoltageSelection;
use hal::timer::{Count, CountDown, TimerExt, TimerUnit};
use hal::watchdog::{TimerInterval, Watchdog, Enable, Disable};

#[entry]
fn main() -> ! {

    // Take the Peripherals
    let p = pac::Peripherals::take().unwrap();

    // Setup the Watchdog
    let mut _watchdog = p.WDT_A.constrain()
        .try_disable().unwrap()
        .try_start(TimerInterval::At31).unwrap()
        .try_feed().unwrap();

    // PCM Configuration with DCDC max. voltage - 48 MHz MCLK operation
    let pcm = p.PCM.constrain()
        .set_core_voltage(CoreVoltageSelection::DcDc)
        .freeze();

    // Get the current Power Mode
    let _pcm_sel = pcm.get_power_mode();

    // Flash Control Config.
    let _flash_control = p.FLCTL.constrain()                                         // Setup Flash
        .set_waitstates(FlashWaitStates::_2)                               // Two wait states -> 48 Mhz Clock
        .freeze();

    let _clock = p.CS.constrain()                                            // Setup CS
        .mclk_dcosource_selection(DCOFrequency::_48MHz, MPrescaler::DIVM_0)  // 48 MHz DCO
        .smclk_prescaler(SMPrescaler::DIVS_1)                                // 24 MHz SMCLK
        .freeze();

    hprintln!("Hello World Example - PCM: {}", _pcm_sel as u32).unwrap();

    let gpio = p.DIO.split();
    let mut p1_0 = gpio.p1_0.into_output();

    let mut tim0 = p.TIMER_A0.constrain().set_clock(_clock);
    let count = Count(10, TimerUnit::Hertz);
    tim0.try_start(count).unwrap();

    loop {
        _watchdog.try_feed().unwrap();
        p1_0.try_toggle().unwrap();
        block!(tim0.try_wait()).unwrap();
    }
}
