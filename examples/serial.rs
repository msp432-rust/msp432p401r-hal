#![no_main]
#![no_std]
#![feature(llvm_asm)]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use msp432p401r as pac;
use panic_halt as _;

use hal::clock::{CsExt, DCOFrequency, MPrescaler, SMPrescaler};
use hal::flash::{FlashExt, FlashWaitStates};
use hal::gpio::{GpioExt, ToggleableOutputPin};
use hal::pcm::{PcmExt, VCoreSel};
use hal::timer::{Count, CountDown, TimerExt, TimerUnit};
use hal::watchdog::{TimerInterval, Watchdog, WDTExt};
use hal::serial::spi;
use msp432p401r_hal as hal;

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();

    let mut watchdog = p.WDT_A.constrain()
        .set_timer_interval(TimerInterval::At31);

    watchdog.try_feed().unwrap();

    let pcm = p.PCM.constrain()
        .set_vcore(VCoreSel::DcdcVcore1)
        .freeze();

    let flash = p.FLCTL.constrain()
        .set_waitstates(FlashWaitStates::_2Ws)
        .freeze();

    let clock = p.CS.constrain()
        .mclk_dcosource_selection(DCOFrequency::_48MHz, MPrescaler::DIVM_0)
        .smclk_prescaler(SMPrescaler::DIVS_1)
        .freeze();

    let gpio = p.DIO.split();

    let p1_0 = gpio.p1_0.into_alternate_primary();
    p1_0.into_alternate_primary();




    loop {
        watchdog.try_feed().unwrap();
        p1_0.try_toggle().unwrap();
        block!(tim0.try_wait()).unwrap();
    }
}
