#![no_main]
#![no_std]
#![feature(llvm_asm)]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use embedded_hal::spi::*;
use msp432p401r as pac;
use nb::block;
use panic_halt as _;

use hal::clock::{CsExt, DCOFrequency, MPrescaler, SMPrescaler};
use hal::flash::{FlashExt, FlashWaitStates};
use hal::gpio::{GpioExt, ToggleableOutputPin};
use hal::pcm::{PcmExt, VCoreSel};
use hal::pmap::{Mapping, PmapExt, PortMap};
use hal::serial::{spi, i2c::Master, I2C};
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

    let _flash_control = p.FLCTL.constrain()
        .set_waitstates(FlashWaitStates::_2)
        .freeze();

    let clock = p.CS.constrain()
        .mclk_dcosource_selection(DCOFrequency::_48MHz, MPrescaler::DIVM_0)
        .smclk_prescaler(SMPrescaler::DIVS_1)
        .freeze();

    let mut timer = p.TIMER_A0.constrain().set_clock(clock);

    let _pmap = p.PMAP.constrain();
    let gpio = p.DIO.split();

    // Master SPI
    let eusci_a1 = p.EUSCI_B0.ucbx_i2coa0
        .into_i2c()
        .with_own_address()
        .master_mode()

    // Setup eUSCI_A1 SPI PINs into proper alternate mode
    gpio.p2_4.into_alternate_primary().remap(Mapping::UCA1STE, true);
    gpio.p2_6.into_alternate_primary().remap(Mapping::UCA1CLK, true);
    gpio.p2_7.into_alternate_primary().remap(Mapping::UCA1SOMI, true);
    gpio.p2_3.into_alternate_primary();

    // let spi_a1 = eusci_a1.init();

    timer.try_start(Count(1, TimerUnit::Seconds)).unwrap();
    let mut led = gpio.p1_0.into_output();

    let mut tx: u8 = 0xCA;
    let mut rx: u8;

    loop {
        watchdog.try_feed().unwrap();
        led.try_toggle().unwrap();
        hprintln!("Sending: {}", tx).unwrap();
        spi_a1.write(tx);
        rx = spi_a3.read();
        hprintln!("Reading: {}", rx).unwrap();
        block!(timer.try_wait()).unwrap();

        if tx == 0xFF {
            tx = 0;
        } else {
            tx = tx + 1;
        }
    }
}
