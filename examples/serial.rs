/* SPI communication example between two eUSCI modules (A1 and A3)

In order to run this example, connect the following PINs:

STE:    P2_0  -> P9_4
CLK:    P2_1  -> P9_5
MISO:   P2_2  -> P9_6
MOSI:   P2_3  -> P9_7

*/

#![no_main]
#![no_std]
#![feature(llvm_asm)]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use embedded_hal::blocking::spi::Write;
use embedded_hal::spi::{FullDuplex, MODE_0, MODE_3};
use msp432p401r as pac;
use nb::block;
use pac::TIMER_A0;
use panic_halt as _;

use hal::clock::{CsExt, DCOFrequency, MPrescaler, SMPrescaler};
use hal::flash::{FlashExt, FlashWaitStates};
use hal::gpio::{GpioExt, ToggleableOutputPin};
use hal::pcm::{PcmExt, VCoreSel};
use hal::serial::{spi, SPI};
use hal::serial::spi::Enabled;
use hal::serial::spi::spia::{SPI_A0, SPI_A1};
use hal::timer::{Count, CountDown, TimerExt, TimerUnit};
use hal::timer::{ClockDefined, TimerConfig};
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

    let clock = p.CS.constrain()
        .mclk_dcosource_selection(DCOFrequency::_48MHz, MPrescaler::DIVM_0)
        .smclk_prescaler(SMPrescaler::DIVS_1)
        .freeze();

    let mut tim0 = p.TIMER_A0.constrain().set_clock(clock);

    let gpio = p.DIO.split();

    // Master SPI
    let eusci_a1 = p.EUSCI_A1.into_spi()
        .master_mode()
        .msb_first()
        .with_clock_source(spi::ClockSource::ACLK)
        .with_mode(MODE_0)
        .with_bit_rate_prescaler(0x02);

    // Setup eUSCI_A1 SPI PINs into proper alternate mode
    gpio.p2_0.into_alternate_primary();
    gpio.p2_1.into_alternate_primary();
    gpio.p2_2.into_alternate_primary();
    gpio.p2_3.into_alternate_primary();

    // Slave SPI
    let eusci_a3 = p.EUSCI_A3.into_spi()
        .slave_mode()
        .msb_first()
        .with_mode(MODE_0);

    // Setup eUSCI_A3 SPI PINs into proper alternate mode
    gpio.p9_4.into_alternate_primary();
    gpio.p9_5.into_alternate_primary();
    gpio.p9_6.into_alternate_primary();
    gpio.p9_7.into_alternate_primary();

    let spi_a1 = eusci_a1.init();
    let spi_a3 = eusci_a3.init();

    tim0.try_start(Count(1, TimerUnit::Seconds)).unwrap();
    let mut data: u8;
    let mut led = gpio.p1_0.into_output();

    loop {
        watchdog.try_feed().unwrap();
        led.try_toggle().unwrap();
        spi_a1.write(0xCA);
        data = spi_a3.read();
        hprintln!("Reading: {}", data);
        block!(tim0.try_wait()).unwrap();
    }
}
