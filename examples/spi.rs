/* SPI communication example between two eUSCI modules (A1 and A3)

In order to run this example, connect the following PINs:

STE:    P2_4  -> P9_4
CLK:    P2_6  -> P9_5
MISO:   P2_7  -> P9_6
MOSI:   P2_3  -> P9_7

*/

#![no_main]
#![no_std]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use msp432p401r as pac;
use msp432p401r_hal as hal;
use panic_halt as _;
use ::nb::block;

use hal::clock::{CsExt, DCOFrequency, MPrescaler, SMPrescaler};
use hal::flash::{FlashExt, FlashWaitStates};
use hal::gpio::{GpioExt, ToggleableOutputPin};
use hal::pcm::{PcmExt, VCoreSel};
use hal::pmap::{Mapping,PmapExt,PortMap};
use hal::serial::{spi, spi::*, SPI};
use hal::timer::{Count, CountDown, TimerExt, TimerUnit};
use hal::watchdog::{TimerInterval, Watchdog, WDTExt};

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();

    let mut watchdog = p.WDT_A.constrain();
    watchdog.set_timer_interval(TimerInterval::At31);
    watchdog.feed().unwrap();

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
    let eusci_a1 = p.EUSCI_A1.into_spi()
        .master_mode()
        .msb_first()
        .with_clock_source(spi::ClockSource::ACLK)
        .with_mode(MODE_0)
        .with_bit_rate_prescaler(0x02);

    // Setup eUSCI_A1 SPI PINs into proper alternate mode
    gpio.p2_4.into_alternate_primary().remap(Mapping::UCA1STE, true);
    gpio.p2_6.into_alternate_primary().remap(Mapping::UCA1CLK,true);
    gpio.p2_7.into_alternate_primary().remap(Mapping::UCA1SOMI, true);
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

    timer.start(Count(1, TimerUnit::Seconds)).unwrap();
    let mut led = gpio.p1_0.into_output();

    let mut tx: u8 = 0xCA;
    let mut rx: u8;

    loop {
        watchdog.feed().unwrap();
        led.toggle().unwrap();
        // @TODO WHY RX[n] = TX[n-1]?
        hprintln!("Sending: {}", tx);
        spi_a1.write(tx);
        rx = spi_a3.read();
        hprintln!("Reading: {}", rx);
        block!(timer.wait()).unwrap();

        if tx == 0xFF {
            tx = 0;
        } else {
            tx = tx + 1;
        }
    }
}