#![no_main]
#![no_std]

use panic_halt as _;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nb::block;

use msp432p401r as pac;
use msp432p401r_hal as hal;

use pac::interrupt;
use hal::{clock::*, flash::*, gpio::*, pcm::*, timer::*, watchdog::*};
use irq::{scoped_interrupts, handler, scope};

static TIM32P: Mutex<RefCell<Option<Timer32Config<ChannelNotDefined, ClockDefined>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {

    // Take the Peripherals
    let p = pac::Peripherals::take().unwrap();

    // Watchdog Config.
    let mut _watchdog = p.WDT_A.constrain();                                 // Setup WatchdogTimer

    _watchdog.set_timer_interval(TimerInterval::At27);
    _watchdog.try_feed().unwrap();

    // PCM Config.
    let _pcm = p.PCM.constrain()                                              // Setup PCM
        .set_vcore(VCoreSel::DcdcVcore1)                                     // Set DCDC Vcore1 -> 48 MHz Clock
        .freeze();

    // Flash Control Config.
    let _flash_control = p.FLCTL.constrain()                                         // Setup Flash
        .set_waitstates(FlashWaitStates::_2)                               // Two wait states -> 48 Mhz Clock
        .freeze();

    let _clock = p.CS.constrain()                                            // Setup CS
        .mclk_dcosource_selection(DCOFrequency::_48MHz, MPrescaler::DIVM_0)  // 48 MHz DCO
        .smclk_prescaler(SMPrescaler::DIVS_1)                                // 24 MHz SMCLK
        .freeze();

    hprintln!("TIMER 32 Example").unwrap();

    let gpio = p.DIO.split();

    // LED 1 - RED
    let mut p1_0 = gpio.p1_0.into_output();
    // LED 2 - RGB
    let mut p2_0 = gpio.p2_0.into_output();
    let mut p2_1 = gpio.p2_1.into_output();
    let mut p2_2 = gpio.p2_2.into_output();

    let mut tim32 = p.TIMER32.constrain().set_clock(_clock);

    let mut count = Count(3, TimerUnit::Seconds);

    p1_0.try_toggle().unwrap();

    // TIM32 Channel 0 - One shot Mode
    tim32.channel0().try_start_oneshot(count).unwrap();

    // TIM32 Channel 1 - Free Running Mode
    tim32.channel1().try_start_freerunning().unwrap();

    let mut ticks = tim32.channel1().get_ticks();
    hprintln!("Ticks 0: 0x{:x?}", ticks).unwrap();

    block!(tim32.channel0().try_wait()).unwrap();

    p1_0.try_toggle().unwrap();

    ticks = tim32.channel1().get_ticks();
    hprintln!("Ticks 1: 0x{:x?}", ticks).unwrap();

    // TIMER32 - Stop Timers and Reset Config
    tim32.channel0().try_cancel().unwrap();
    tim32.channel1().try_cancel().unwrap();

    // TIM32 Channel 0 - Periodic Mode - Blocking
    count = Count(10, TimerUnit::Hertz);
    tim32.channel0().try_start(count).unwrap();

    // TIM32 Channel 1 - Periodic Mode - Interrupt
    count = Count(1, TimerUnit::Hertz);
    tim32.channel1().enable_interrupt().try_start(count).unwrap();

    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::interrupt::T32_INT2_IRQ);
        cortex_m::interrupt::enable();
    }

    let mut led_state: u8 = 0;

    cortex_m::interrupt::free(move |cs| {
        TIM32P.borrow(cs).replace(Some(tim32));
    });

    handler!(
        int32_1 = move || {

            match led_state {
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
            }

            led_state += 1;

            if led_state == 7 {
                led_state = 0;
            }

            cortex_m::interrupt::free(|cs| {
                let mut tim = TIM32P.borrow(cs).borrow_mut();
                tim.as_mut().unwrap().channel1().clear_interrupt_pending_bit();
            });
        }
    );

    scope(|scope| {
        scope.register(Interrupts::T32_INT2_IRQ, int32_1);

        loop {
            _watchdog.try_feed().unwrap();
            p1_0.try_toggle().unwrap();
            block!({
                cortex_m::interrupt::free(|cs| {
                    let mut tim = TIM32P.borrow(cs).borrow_mut();
                    tim.as_mut().unwrap().channel0().try_wait()
                })
            }).unwrap();
        }
    });

    loop {
        _watchdog.try_feed().unwrap();
        continue;
    }
}

scoped_interrupts! {
    #[allow(non_camel_case_types)]
    enum Interrupts {
        T32_INT2_IRQ,
    }

    use #[interrupt];
}
