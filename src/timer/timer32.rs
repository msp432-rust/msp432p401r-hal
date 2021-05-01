//! HAL library for Timer module (Timer32) - MSP432P401R
pub use embedded_hal::timer::{Cancel, CountDown, Periodic};
use pac::TIMER32;
use core::marker::PhantomData;
use crate::common::{Split, NotDefined, Defined, Error};
use crate::clock::Clocks;
use crate::timer::time::{TimeCount, Hertz, MilliSeconds};

const MAX_PRESCALER32: u32 = 0x0100;

pub struct Channel0 {
    _marker: PhantomData<*const ()>,
}
pub struct Channel1 {
    _marker: PhantomData<*const ()>,
}

pub trait OneShot {
    type Error;
    type Time;
    fn try_start_oneshot<T>(&mut self, count: T) -> Result<(), Self::Error>
        where
            T: Into<Self::Time>;
}

pub trait FreeRunning {
    type Error;
    fn try_start_freerunning(&mut self) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, Copy)]
enum ClockSourcePrescaler32 {
    _1   =  1,
    _16  =  16,
    _256 = 256,
}

pub struct Timer32Control<C, State> {
    clocks: Clocks,
    _state: State,
    channel: PhantomData<C>,
    tim: Option<*const TIMER32>,
}

pub struct Parts<State> {
    pub channel0: Timer32Control<Channel0, State>,
    pub channel1: Timer32Control<Channel1, State>,
}

impl Split for TIMER32 {
    type Parts = Parts<NotDefined>;

    fn split(self) -> Self::Parts {
        Parts::<NotDefined>::new(self)
    }
}

impl Parts<NotDefined> {
    fn new(timer: TIMER32 ) -> Parts::<NotDefined> {
        let hz: Hertz = Hertz(0);
        let clock: Clocks = Clocks{aclk: hz, mclk: hz, hsmclk: hz, smclk: hz, bclk: hz };

        Parts::<NotDefined> {
            channel0: Timer32Control::<Channel0, NotDefined> {
                clocks: clock,
                _state: NotDefined,
                channel: PhantomData,
                tim: Some(&timer),
            },
            channel1: Timer32Control::<Channel1, NotDefined> {
                clocks: clock,
                _state: NotDefined,
                channel: PhantomData,
                tim: Some(&timer),
            },
        }
    }

    pub fn set_clock(self, clock: Clocks) -> Parts<Defined> {
        Parts::<Defined> {
            channel0: Timer32Control::<Channel0, Defined> {
                clocks: clock,
                _state: Defined,
                channel: PhantomData,
                tim: self.channel0.tim,
            },
            channel1: Timer32Control::<Channel1, Defined> {
                clocks: clock,
                _state: Defined,
                channel: PhantomData,
                tim: self.channel1.tim,
            },
        }
    }
}

macro_rules! timer32 {
    ($($Channeli:ident, $t32controli:ident, $t32intclri:ident, $t32misi:ident, $t32loadi:ident, $t32risi:ident, $t32valuei:ident),*) => {
        $(
            impl Timer32Control<$Channeli, Defined> {

                #[inline]
                pub fn update_clock(&mut self, clocks: Clocks) -> &mut Self {
                    self.clocks = clocks;
                    self
                }

                #[inline]
                pub fn get_ticks(&self) -> u32 {
                    unsafe {
                        (*self.tim.unwrap()).$t32valuei.read().bits()
                    }
                }

                #[inline]
                pub fn enable_interrupt(&mut self) -> &mut Self {
                    unsafe {
                        (*self.tim.unwrap()).$t32controli.modify(|_, w| w.ie().ie_1());
                    }
                    self
                }

                #[inline]
                pub fn disable_interrupt(&mut self) -> &mut Self {
                    unsafe {
                        (*self.tim.unwrap()).$t32controli.modify(|_, w| w.ie().ie_0());
                    }
                    self
                }

                #[inline]
                pub fn interrupt_enabled(&self) -> bool {
                    unsafe {
                        (*self.tim.unwrap()).$t32controli.read().ie().is_ie_1() == true
                    }
                }

                #[inline]
                pub fn clear_interrupt_pending_bit(&mut self) -> &mut Self {
                    unsafe {
                        (*self.tim.unwrap()).$t32intclri.write(|w| w.intclr().bits(0x01));
                    }
                    self
                }

                #[inline]
                pub fn check_interrupt(&self) -> bool {
                    unsafe{
                        (*self.tim.unwrap()).$t32misi.read().bits() & 0x01 != 0
                    }
                }

                #[inline]
                fn stop_timer(&mut self) -> &mut Self {
                    unsafe {
                        (*self.tim.unwrap()).$t32controli.modify(|_, w| w.enable().enable_0());
                    }
                    self
                }

                #[inline]
                fn timer_wrapped(&mut self) -> bool {
                    if(unsafe { (*self.tim.unwrap()).$t32controli.read().mode().is_mode_0() ||
                       (*self.tim.unwrap()).$t32risi.read().raw_ifg().bits() }) {
                       self.clear_interrupt_pending_bit();
                       true
                    } else {
                        false
                    }
                }

                #[inline]
                fn timer_running(&self) -> bool {
                    unsafe {
                        (*self.tim.unwrap()).$t32controli.read().enable().is_enable_1() == true
                    }
                }

                #[inline]
                fn setup_timer(&self, count: TimeCount) -> bool {

                    let max_period: u64 = (MAX_PRESCALER32 as u64 * u32::MAX as u64) ;
                    let count_ratio : u64;

                    if count.is_frequency() {
                        let frequency: u32 = u32::from(Hertz::from(count));
                        count_ratio = (self.clocks.mclk.0 / frequency) as u64;
                    } else {
                        let period: u32 = u32::from(MilliSeconds::from(count));
                        count_ratio = (period *(self.clocks.mclk.0/1000)) as u64;
                    }

                    if(count_ratio < max_period) {
                        let mut min_prescaler = (count_ratio / u32::MAX as u64) as u8;

                        // In period, prescaler need to be above the min_prescaler value
                        if count.is_period(){
                            min_prescaler = min_prescaler + 1;
                        }

                        let real_prescaler = self.get_prescaler(min_prescaler);
                        let tick_count = (count_ratio / real_prescaler as u64) as u32;
                        self.set_count(tick_count);
                        true
                    } else {
                        false
                    }
                }

                #[inline]
                fn set_count(&self, count: u32) {
                    unsafe {
                        (*self.tim.unwrap()).$t32loadi.modify(|_, w| w.bits(count));
                    }
                }

                #[inline]
                fn get_prescaler(&self, min_prescaler: u8) -> ClockSourcePrescaler32 {
                    use ClockSourcePrescaler32::*;
                    match min_prescaler {
                        0..=1 => self.setup_prescaler(_1),
                        2..=16 => self.setup_prescaler(_16),
                        _ => self.setup_prescaler(_256),
                    }
                }

                #[inline]
                fn setup_prescaler(&self, prescaler: ClockSourcePrescaler32) -> ClockSourcePrescaler32 {
                    use ClockSourcePrescaler32::*;
                    match prescaler {
                          _1  => unsafe {(*self.tim.unwrap()).$t32controli.modify(|_, w| w.prescale().prescale_0())},
                         _16  => unsafe {(*self.tim.unwrap()).$t32controli.modify(|_, w| w.prescale().prescale_1())},
                        _256  => unsafe {(*self.tim.unwrap()).$t32controli.modify(|_, w| w.prescale().prescale_2())},
                    };
                    prescaler
                }

                #[inline]
                fn start_timer(&self) {
                    unsafe {
                        (*self.tim.unwrap()).$t32controli.modify(|_, w|
                            w.enable().enable_1()
                            .mode().mode_1()
                            .size().size_1()
                            .oneshot().oneshot_0()
                        );
                    }
                }

                #[inline]
                fn start_oneshot(&self) {
                    unsafe {
                        (*self.tim.unwrap()).$t32controli.modify(|_, w|
                            w.enable().enable_1()
                            .mode().mode_1()
                            .size().size_1()
                            .oneshot().oneshot_1()
                        );
                    }
                }

                #[inline]
                fn start_freerunning(&self) {
                    unsafe {
                        (*self.tim.unwrap()).$t32controli.modify(|_, w|
                            w.enable().enable_1()
                            .mode().mode_0()
                            .size().size_1()
                            .oneshot().oneshot_0()
                        );
                    }
                }
            }

            impl CountDown for Timer32Control<$Channeli, Defined>{
                type Error = Error;
                type Time = TimeCount;

                fn try_start <T>(&mut self, count: T) -> Result<(), Self::Error>
                where
                    T: Into<Self::Time> {
                    if(!self.timer_running()) {

                        if(self.setup_timer(count.into())) {
                            self.start_timer();
                            Ok(())
                        } else {
                            Err(Error::Unreachable)
                        }

                    } else {
                        Err(Error::Enabled)
                    }
                }

                fn try_wait(&mut self) -> nb::Result<(), Self::Error> {
                   if(self.timer_wrapped()) {
                        Ok(())
                   } else {
                        Err(nb::Error::WouldBlock)
                   }
                }
            }

            impl Cancel for Timer32Control<$Channeli, Defined> {
                fn try_cancel(&mut self) -> Result<(), Self::Error> {
                    if(self.timer_running()) {
                        self.stop_timer();
                        Ok(())
                    } else {
                        Err(Error::Disabled)
                    }
                }
            }

            impl Periodic for Timer32Control<$Channeli, Defined> {}

            impl OneShot for Timer32Control<$Channeli, Defined> {
                type Error = Error;
                type Time = TimeCount;

                fn try_start_oneshot <T>(&mut self, count: T) -> Result<(), Self::Error>
                    where
                        T: Into<Self::Time> {
                            if(!self.timer_running()) {
                                if(self.setup_timer(count.into())) {
                                    self.start_oneshot();
                                    Ok(())
                            } else {
                                Err(Error::Unreachable)
                        }
                    } else {
                        Err(Error::Enabled)
                    }
                }
            }

            impl FreeRunning for Timer32Control<$Channeli, Defined> {
                type Error = Error;

                fn try_start_freerunning(&mut self) -> Result<(), Self::Error> {
                            if(!self.timer_running()) {
                                self.set_count(u32::MAX);
                                self.start_freerunning();
                                Ok(())
                    } else {
                        Err(Error::Enabled)
                    }
                }
            }
        )*
    };
}

timer32! {
    Channel0, t32control1, t32intclr1, t32mis1, t32load1, t32ris1, t32value1,
    Channel1, t32control2, t32intclr2, t32mis2, t32load2, t32ris2, t32value2
}