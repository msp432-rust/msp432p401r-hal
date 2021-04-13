#![no_main]
#![no_std]
#![feature(llvm_asm)]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use embedded_hal::spi::{MODE_1, FullDuplex};
use msp432p401r as pac;
use panic_halt as _;

use hal::clock::{CsExt, DCOFrequency, MPrescaler, SMPrescaler};
use hal::flash::{FlashExt, FlashWaitStates};
use hal::gpio::{GpioExt, ToggleableOutputPin};
use hal::pcm::{PcmExt, VCoreSel};
use hal::serial::{spi, SPI};
use hal::timer::{Count, CountDown, TimerExt, TimerUnit};
use hal::watchdog::{TimerInterval, Watchdog, WDTExt};
use msp432p401r_hal as hal;

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();

    let mut watchdog = p.WDT_A.constrain();
    watchdog.set_timer_interval(TimerInterval::At31);
    watchdog.try_feed().unwrap();

    let _pcm = p.PCM.constrain()
        .set_vcore(VCoreSel::DcdcVcore1)
        .freeze();

    let _flash = p.FLCTL.constrain()
        .set_waitstates(FlashWaitStates::_2Ws)
        .freeze();

    let _clock = p.CS.constrain()
        .mclk_dcosource_selection(DCOFrequency::_48MHz, MPrescaler::DIVM_0)
        .smclk_prescaler(SMPrescaler::DIVS_1)
        .freeze();

    let gpio = p.DIO.split();

    let spi_a0 = p.EUSCI_A0.into_spi()
        .with_ports(gpio.p1_0.into_alternate_primary(),
                    gpio.p1_1.into_alternate_primary(),
                    gpio.p1_2.into_alternate_primary(),
                    gpio.p1_3.into_alternate_primary())
        .with_clock_source(spi::ClockSource::ACLK)
        .with_mode(MODE_1)
        .master_mode()
        .lsb_first()
        .init();

    loop {
        watchdog.try_feed().unwrap();
        // spi_a0.try_read().unwrap()
    }
}
