#![no_main]
#![no_std]
#![feature(llvm_asm)]

use panic_halt as _;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nb::block;

use msp432p401r as pac;
use msp432p401r_hal as hal;

use hal::common::{Constrain, Split};
use hal::clock::{DCOFrequency, MPrescaler, SMPrescaler};
use hal::flash::{FlashWaitStates};
use hal::gpio::ToggleableOutputPin;
use hal::pcm::CoreVoltageSelection;
use hal::timer::{time::TimerUnit, timer16::CountDown};
use hal::watchdog::{Options, ClockSource, TimerInterval, Watchdog, Enable, Disable};
use pac::Peripherals;

#[entry]
fn main() -> ! {

    // Take the Peripherals
    let p: Peripherals = Peripherals::take().unwrap();

    // Setup the Watchdog - Disable the WDT to configure some parameters.
    let mut watchdog = p.WDT_A.constrain()
        .try_disable().unwrap()
        .try_start(Options(ClockSource::SMCLK,TimerInterval::At31)).unwrap();

    // PCM Configuration with DCDC max. voltage - 48 MHz MCLK operation
    let pcm = p.PCM.constrain()
        .set_core_voltage(CoreVoltageSelection::DcDc)
        .freeze();

    // Get the current Power Mode
    let _power = pcm.get_power_mode();

    // Get the current Core Voltage
    let _voltage = pcm.get_core_voltage();

    // Setup Flash Control - Two wait states for 48 MHz.
    let _flash_control = p.FLCTL.constrain()
        .set_waitstates(FlashWaitStates::_2)
        .freeze();

    // Setup the Clock Source - MCLK: 48 MHz DCO | SMCLK: 24 MHz
    let _clock = p.CS.constrain()
        .mclk_dcosource_selection(DCOFrequency::_48MHz, MPrescaler::DIVM_0)
        .smclk_prescaler(SMPrescaler::DIVS_1)
        .freeze();

    hprintln!("Hello World Example").unwrap();
    hprintln!("Power Mode: {:?}", _power).unwrap();
    hprintln!("Core Voltage: {:?}", _voltage).unwrap();

    let gpio = p.DIO.split();
    let mut p1_0 = gpio.p1_0.into_output();

    let mut tim0 = p.TIMER_A0.constrain().set_clock(_clock);
    tim0.try_start(10.hertz()).unwrap();

    loop {
        watchdog.try_feed().unwrap();
        p1_0.try_toggle().unwrap();
        block!(tim0.try_wait()).unwrap();
    }
}
