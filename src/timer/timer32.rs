//! HAL library for Timer module (Timer32) - MSP432P401R
pub use embedded_hal::timer::Periodic;
pub use embedded_hal::timer::nb::{Cancel, CountDown};

use pac::TIMER32;
use core::marker::PhantomData;
use core::mem::transmute;

use crate::clock::Clocks;
use crate::time::Hertz;
use crate::timer::*;

const MAX_PRESCALER32: u32 = 0x0100;

unsafe impl Send for ChannelNotDefined {}

pub struct ChannelNotDefined {
    _marker: PhantomData<*const ()>,
}
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

pub struct Timer32Config<C, S: State> {
    clocks: Clocks,
    tim: TIMER32,
    _state: S,
    channel: PhantomData<C>,
}

pub trait Timer32Ext {
    type Output;
    fn constrain(self) -> Self::Output;
}

impl Timer32Ext for TIMER32 {

    type Output = Timer32Config <ChannelNotDefined, ClockNotDefined>;

    fn constrain(self) -> Self::Output {
        Timer32Config::<ChannelNotDefined,ClockNotDefined>::tim32(self)
    }
}

impl Timer32Config <ChannelNotDefined, ClockNotDefined> {

    fn tim32(timer: TIMER32 ) -> Timer32Config <ChannelNotDefined, ClockNotDefined> {

        let hz: Hertz = Hertz(0);
        let clock: Clocks = Clocks{aclk: hz, mclk: hz, hsmclk: hz, smclk: hz, bclk: hz };

        Timer32Config {
            clocks: clock,
            tim: timer,
            _state: ClockNotDefined,
            channel: PhantomData,
        }
    }

    pub fn set_clock(self, clock: Clocks) -> Timer32Config <ChannelNotDefined, ClockDefined> {
        Timer32Config {
            clocks: clock,
            tim: self.tim,
            _state: ClockDefined,
            channel: PhantomData,
        }
    }
}

impl <C>Timer32Config <C, ClockDefined> {
    pub fn channel0(&mut self) -> &mut Timer32Config::<Channel0, ClockDefined> {
        unsafe {
            transmute::<&mut Self, &mut Timer32Config <Channel0, ClockDefined>>(self)
        }
    }

    pub fn channel1(&mut self) -> &mut Timer32Config::<Channel1, ClockDefined> {
        unsafe {
            transmute::<&mut Self, &mut Timer32Config <Channel1, ClockDefined>>(self)
        }
    }
}

macro_rules! timer32 {
    ($($Channeli:ident, $t32controli:ident, $t32intclri:ident, $t32misi:ident, $t32loadi:ident, $t32risi:ident, $t32valuei:ident),*) => {
        $(

            impl Timer32Config <$Channeli, ClockDefined> {

                #[inline]
                pub fn update_clock(&mut self, clocks: Clocks) -> &mut Self {
                    self.clocks = clocks;
                    self
                }

                #[inline]
                pub fn get_ticks(&self) -> u32 {
                    self.tim.$t32valuei.read().bits()
                }

                #[inline]
                pub fn enable_interrupt(&mut self) -> &mut Self {
                    self.tim.$t32controli.modify(|_, w| w.ie().ie_1());
                    self
                }

                #[inline]
                pub fn disable_interrupt(&mut self) -> &mut Self {
                    self.tim.$t32controli.modify(|_, w| w.ie().ie_0());
                    self
                }

                #[inline]
                pub fn interrupt_enabled(&self) -> bool {
                    self.tim.$t32controli.read().ie().is_ie_1() == true
                }

                #[inline]
                pub fn clear_interrupt_pending_bit(&mut self) -> &mut Self {
                    self.tim.$t32intclri.write(|w| unsafe {
                        w.intclr().bits(0x01)
                    });

                    self
                }

                #[inline]
                pub fn check_interrupt(&self) -> bool {
                    self.tim.$t32misi.read().bits() & 0x01 != 0
                }

                #[inline]
                fn stop_timer(&mut self) -> &mut Self {
                    self.tim.$t32controli.modify(|_, w| w.enable().enable_0());
                    self
                }

                #[inline]
                fn timer_wrapped(&mut self) -> bool {
                    if(self.tim.$t32controli.read().mode().is_mode_0() ||
                       self.tim.$t32risi.read().raw_ifg().bit()) {
                       self.clear_interrupt_pending_bit();
                       true
                    } else {
                        false
                    }
                }

                #[inline]
                fn timer_running(&self) -> bool {
                    self.tim.$t32controli.read().enable().is_enable_1() == true
                }

                #[inline]
                fn setup_timer(&self, count: Count) -> bool {
                    use TimerUnit::*;
                    match count.1 {
                        Hertz | Milliseconds => self.setup_count(count),
                        Kilohertz => self.setup_count(Count(count.0 * 1000, Hertz)),
                        Seconds => self.setup_count(Count(count.0 * 1000, Milliseconds)),
                    }
                }

                #[inline]
                fn setup_count(&self, count: Count) -> bool {

                    let max_period: u64 = (MAX_PRESCALER32 as u64 * u32::MAX as u64) ;
                    let count_ratio : u64;

                    if count.1 == TimerUnit::Hertz {
                        let frequency = count.0;
                        count_ratio = (self.clocks.mclk.0 / frequency) as u64;
                    } else {
                        let period = count.0;
                        count_ratio = (period*(self.clocks.mclk.0/1000)) as u64;
                    }

                    if(count_ratio < max_period) {
                        let mut min_prescaler = (count_ratio / u32::MAX as u64) as u8;

                        // In period, prescaler need to be above the min_prescaler value
                        if count.1 != TimerUnit::Hertz {
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
                    self.tim.$t32loadi.modify(|_, w| unsafe {
                        w.bits(count)
                    });
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
                          _1  => self.tim.$t32controli.modify(|_, w| w.prescale().prescale_0()),
                         _16  => self.tim.$t32controli.modify(|_, w| w.prescale().prescale_1()),
                        _256  => self.tim.$t32controli.modify(|_, w| w.prescale().prescale_2()),
                    };

                    prescaler
                }

                #[inline]
                fn start_timer(&self) {
                    self.tim.$t32controli.modify(|_, w|
                        w.enable().enable_1()
                        .mode().mode_1()
                        .size().size_1()
                        .oneshot().oneshot_0()
                    );
                }

                #[inline]
                fn start_oneshot(&self) {
                    self.tim.$t32controli.modify(|_, w|
                        w.enable().enable_1()
                        .mode().mode_1()
                        .size().size_1()
                        .oneshot().oneshot_1()
                    );
                }

                #[inline]
                fn start_freerunning(&self) {
                    self.tim.$t32controli.modify(|_, w|
                        w.enable().enable_1()
                        .mode().mode_0()
                        .size().size_1()
                        .oneshot().oneshot_0()
                    );
                }
            }

            impl CountDown for Timer32Config <$Channeli, ClockDefined>{
                type Error = Error;
                type Time = Count;

                fn start <T>(&mut self, count: T) -> Result<(), Self::Error>
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

                fn wait(&mut self) -> nb::Result<(), Self::Error> {
                   if(self.timer_wrapped()) {
                        Ok(())
                   } else {
                        Err(nb::Error::WouldBlock)
                   }
                }
            }

            impl Cancel for Timer32Config <$Channeli, ClockDefined> {
                fn cancel(&mut self) -> Result<(), Self::Error> {
                    if(self.timer_running()) {
                        self.stop_timer();
                        Ok(())
                    } else {
                        Err(Error::Disabled)
                    }
                }
            }

            impl Periodic for Timer32Config <$Channeli, ClockDefined> {}

            impl OneShot for Timer32Config <$Channeli, ClockDefined> {

                type Error = Error;
                type Time = Count;

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

            impl FreeRunning for Timer32Config <$Channeli, ClockDefined> {

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