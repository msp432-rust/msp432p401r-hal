#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m::Peripherals;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nb::block;

use msp432p401r as pac;
use msp432p401r_hal as hal;

use hal::common::{Constrain, Split};
use hal::clock::{DCOFrequency, MPrescaler, SMPrescaler};
use hal::flash::{FlashWaitStates};
use hal::gpio::{ToggleableOutputPin, OutputPin};
use hal::pcm::CoreVoltageSelection;
use hal::timer::{time::TimerUnit, timer32::{Cancel, OneShot, CountDown, FreeRunning}};
use hal::watchdog::{Options, ClockSource, TimerInterval, Watchdog, Enable, Disable};
use pac::interrupt;

static mut LED_STATE: u8 = 0;

#[entry]
fn main() -> ! {

    // Take the Peripherals
    let p = pac::Peripherals::take().unwrap();
    let mut cp = Peripherals::take().unwrap();

    // Setup the Watchdog - Disable the WDT to configure some parameters.
    let mut watchdog = p.WDT_A.constrain()
        .try_disable().unwrap()
        .try_start(Options(ClockSource::SMCLK,TimerInterval::At31)).unwrap();

    // PCM Configuration with DCDC max. voltage - 48 MHz MCLK operation
    let _pcm = p.PCM.constrain()
        .set_core_voltage(CoreVoltageSelection::DcDc)
        .freeze();

    // Setup Flash Control - Two wait states for 48 MHz.
    let _flash_control = p.FLCTL.constrain()
        .set_waitstates(FlashWaitStates::_2)
        .freeze();

    // Setup the Clock Source - MCLK: 48 MHz DCO | SMCLK: 24 MHz
    let _clock = p.CS.constrain()
        .mclk_dcosource_selection(DCOFrequency::_48MHz, MPrescaler::DIVM_0)
        .smclk_prescaler(SMPrescaler::DIVS_1)
        .freeze();

    hprintln!("TIMER 32 Example").unwrap();

    let gpio = p.DIO.split();

    // LED 1 - RED
    let mut p1_0 = gpio.p1_0.into_output();
    // LED 2 - RGB
    let mut p2_0 = gpio.p2_0.into_output();
    let mut p2_1 = gpio.p2_1.into_output();
    let mut p2_2 = gpio.p2_2.into_output();

    let timer32 = p.TIMER32.split().set_clock(_clock);
    let mut timer0 = timer32.channel0;
    let mut timer1 = timer32.channel1;

    p1_0.try_toggle().unwrap();

    // TIM32 Channel 0 - One shot Mode
    timer0.try_start_oneshot(3.seconds()).unwrap();

    // TIM32 Channel 1 - Free Running Mode
    timer1.try_start_freerunning().unwrap();

    let mut ticks = timer1.get_ticks();
    hprintln!("Ticks 0: 0x{:x?}", ticks).unwrap();

    block!(timer0.try_wait()).unwrap();

    p1_0.try_toggle().unwrap();

    ticks = timer1.get_ticks();
    hprintln!("Ticks 1: 0x{:x?}", ticks).unwrap();

    // TIMER32 - Stop Timers and Reset Config
    timer0.try_cancel().unwrap();
    timer1.try_cancel().unwrap();

    // TIM32 Channel 0 - Periodic Mode - Blocking
    timer0.try_start(10.hertz()).unwrap();

    // TIM32 Channel 1 - Periodic Mode - Interrupt
    timer1.enable_interrupt().try_start(1.hertz()).unwrap();

    cortex_m::interrupt::free(|_| {
        unsafe{
            cortex_m::peripheral::NVIC::unmask(pac::interrupt::T32_INT2_IRQ);
            cp.SYST.enable_interrupt();
        }
    });

    let mut led_state: u8 = 0xff;

    loop {
        watchdog.try_feed().unwrap();
        p1_0.try_toggle().unwrap();

        block!({
            if led_state != unsafe{LED_STATE} {
                match unsafe{LED_STATE} {
                    0 => {
                        p2_0.try_set_high().unwrap();
                        p2_1.try_set_low().unwrap();
                        p2_2.try_set_low().unwrap();
                    },
                    1 => {
                        p2_1.try_set_high().unwrap();
                    },
                    2 => {
                        p2_0.try_set_low().unwrap();
                    },
                    3 => {
                        p2_2.try_set_high().unwrap();
                    },
                    4 => {
                        p2_0.try_set_high().unwrap();
                    },
                    5 => {
                        p2_1.try_set_low().unwrap();
                    },
                    _ => {
                        p2_0.try_set_low().unwrap();
                    },
                };
                led_state = unsafe{LED_STATE};
            }

            if timer1.check_interrupt() {
                timer1.clear_interrupt_pending_bit();

                cortex_m::interrupt::free(|_| {
                    unsafe{
                        cortex_m::peripheral::NVIC::unmask(pac::interrupt::T32_INT2_IRQ);
                    }
                });
            };

            timer0.try_wait()
        }).unwrap();
    }
}

#[interrupt]
fn T32_INT2_IRQ() -> () {
    cortex_m::interrupt::free(|_| {
        unsafe {
            LED_STATE += 1;

            if LED_STATE == 7 {
                LED_STATE = 0;
            }
            cortex_m::peripheral::NVIC::unpend(pac::interrupt::T32_INT2_IRQ);
            cortex_m::peripheral::NVIC::mask(pac::interrupt::T32_INT2_IRQ);
        }
    });
}