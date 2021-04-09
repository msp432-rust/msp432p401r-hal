//! HAL library for Timer module (Timer32 and TimerA) - MSP432P401R
pub use embedded_hal::timer::{Cancel, CountDown, Periodic};

/// ********************************            TIMER A            *********************************
use pac::TIMER_A0;
use pac::TIMER_A1;
use pac::TIMER_A2;
use pac::TIMER_A3;

use crate::clock::Clocks;
use crate::time::Hertz;

use ClockSource::*;
use ClockSourcePrescaler::*;

pub trait State {}
pub struct ClockNotDefined;
pub struct ClockDefined;

impl State for ClockNotDefined {}
impl State for ClockDefined {}

const MAX_PRESCALER: u32 = 0x0040;
const MAX_COUNT: u32 = 0xFFFF;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum TimerUnit {
    Hertz,
    Kilohertz,
    Milliseconds,
    Seconds,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    Disabled,
    Enabled,
    Unreachable,
}

#[derive(Debug, Clone, Copy)]
pub enum ClockSource {
    ExternalTxclk,
    Aclk,
    Smclk,
    InvertedExternalTxclk,
}

#[derive(Debug, Clone, Copy)]
enum ClockSourcePrescaler {
    _1 =  1,
    _2 =  2,
    _3 =  3,
    _4 =  4,
    _5 =  5,
    _6 =  6,
    _7 =  7,
    _8 =  8,
    _10 = 10,
    _12 = 12,
    _14 = 14,
    _16 = 16,
    _20 = 20,
    _24 = 24,
    _28 = 28,
    _32 = 32,
    _40 = 40,
    _48 = 48,
    _56 = 56,
    _64 = 64,
}

#[derive(PartialEq, PartialOrd)]
pub struct Count(pub u32, pub TimerUnit);

pub struct TimerConfig<T, S: State> {
    clocks: Clocks,
    tim: T,
    _state: S,
    count: u16,
}

pub trait TimerExt {
    type Output;
    fn constrain(self) -> Self::Output;
}

macro_rules! timer {
    ($($TIMER:ident, $tim:ident),*) => {
        $(
            impl TimerExt for $TIMER {

                type Output = TimerConfig<$TIMER, ClockNotDefined>;

                fn constrain(self) -> Self::Output {
                    TimerConfig::<$TIMER, ClockNotDefined>::$tim(self)
                }
            }

            impl TimerConfig <$TIMER, ClockNotDefined> {

                fn $tim(timer: $TIMER) -> TimerConfig <$TIMER, ClockNotDefined>{

                    let hz: Hertz = Hertz(0);
                    let clock: Clocks = Clocks{aclk: hz, mclk: hz, hsmclk: hz, smclk: hz, bclk: hz };

                    TimerConfig {
                        clocks: clock,
                        tim: timer,
                        _state: ClockNotDefined,
                        count: 0,
                    }
                }

                pub fn set_clock(self, clock: Clocks) -> TimerConfig <$TIMER, ClockDefined> {
                    TimerConfig {
                        clocks: clock,
                        tim: self.tim,
                        _state: ClockDefined,
                        count: 0,
                    }
                }
            }

            impl TimerConfig <$TIMER, ClockDefined> {

                #[inline]
                pub fn update_clock(&mut self, clocks: Clocks) -> &mut Self {
                    self.clocks = clocks;
                    self
                }

                #[inline]
                pub fn enable_interrupt(&mut self) -> &mut Self{
                    self.tim.tax_ctl.modify(|_, w| w.taie().taie_1());
                    self
                }

                #[inline]
                pub fn disable_interrupt(&mut self) -> &mut Self {
                    self.tim.tax_ctl.modify(|_, w| w.taie().taie_0());
                    self
                }

                #[inline]
                pub fn interrupt_enabled(&self) -> bool {
                    self.tim.tax_ctl.read().taie().is_taie_1() == true
                }

                #[inline]
                pub fn clear_interrupt_pending_bit(&mut self) -> &mut Self {
                    self.tim.tax_ctl.modify(|_, w| w.taifg().clear_bit());
                    self
                }

                #[inline]
                pub fn check_interrupt(&self) -> bool {
                    self.tim.tax_ctl.read().taifg().is_taifg_1() == true
                }

                #[inline]
                fn stop_timer(&mut self) {
                    self.tim.tax_ctl.modify(|_, w| w.mc().mc_0());
                }

                #[inline]
                fn timer_wrapped(&mut self) -> bool {
                    let val: u16 = self.tim.tax_r.read().bits();
                        if(self.count > val && self.count >= self.tim.tax_ccr[0].read().bits() - 1) {
                            self.count = 0;
                            true
                        } else {
                            self.count = val;
                            false
                        }
                }

                #[inline]
                fn timer_running(&self) -> bool {
                    self.tim.tax_ctl.read().mc().is_mc_0() == false
                }

                #[inline]
                fn clear_timer(&mut self) {
                    self.tim.tax_ctl.modify(|_, w| w.taclr().set_bit());
                    self.count = 0;
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

                    let max_period = MAX_PRESCALER * MAX_COUNT;
                    let aclk_ratio: u32;
                    let smclk_ratio : u32;
                    let mut count_ratio : u32  = 0;
                    let mut clock_source : ClockSource = Smclk;

                    if count.0.checked_mul(self.clocks.aclk.0) == None {
                        return false;
                    }

                    if count.1 == TimerUnit::Hertz {
                        let frequency = count.0;
                        aclk_ratio = self.clocks.aclk.0 / frequency;
                        smclk_ratio = self.clocks.smclk.0 / frequency;
                    } else {
                        let period = count.0;
                        aclk_ratio = (period*self.clocks.aclk.0)/1000;
                        smclk_ratio = period*(self.clocks.smclk.0/1000);
                    }

                    if(smclk_ratio < max_period) {
                        count_ratio = smclk_ratio;
                        clock_source = Smclk;
                    } else if (aclk_ratio < max_period) {
                        count_ratio = aclk_ratio;
                        clock_source = Aclk;
                    }

                    if count_ratio != 0 {
                        let mut min_prescaler = (count_ratio / MAX_COUNT) as u8;

                        // In period, prescaler need to be above the min_prescaler value
                        if count.1 != TimerUnit::Hertz {
                            min_prescaler = min_prescaler + 1;
                        }

                        let real_prescaler = self.get_prescaler(min_prescaler);
                        let tick_count = (count_ratio / real_prescaler as u32) as u16;
                        self.set_clock_source(clock_source);
                        self.set_count(tick_count);
                        true
                    } else {
                        false
                    }
                }

                fn set_clock_source(&self, source: ClockSource) {
                  self.tim.tax_ctl.modify(|_, w| match source {
                      ExternalTxclk         => w.tassel().tassel_0(),
                      Aclk                  => w.tassel().tassel_1(),
                      Smclk                 => w.tassel().tassel_2(),
                      InvertedExternalTxclk => w.tassel().tassel_3(),
                    });
                }

                #[inline]
                fn set_count(&self, count: u16) {
                    self.tim.tax_ccr[0].modify(|r, w| unsafe {
                        w.bits(r.bits() | count)
                    });
                }

                #[inline]
                fn get_prescaler(&self, min_prescaler: u8) -> ClockSourcePrescaler {

                    match min_prescaler {
                        0..=1 => self.setup_prescaler(_1),
                        2 => self.setup_prescaler(_2),
                        3 => self.setup_prescaler(_3),
                        4 => self.setup_prescaler(_4),
                        5 => self.setup_prescaler(_5),
                        6 => self.setup_prescaler(_6),
                        7 => self.setup_prescaler(_7),
                        8 => self.setup_prescaler(_8),
                        9..=10 => self.setup_prescaler(_10),
                        11..=12 => self.setup_prescaler(_12),
                        13..=14 => self.setup_prescaler(_14),
                        15..=16 => self.setup_prescaler(_16),
                        17..=20 => self.setup_prescaler(_20),
                        21..=24 => self.setup_prescaler(_24),
                        25..=28 => self.setup_prescaler(_28),
                        29..=32 => self.setup_prescaler(_32),
                        33..=40 => self.setup_prescaler(_40),
                        41..=48 => self.setup_prescaler(_48),
                        49..=56 => self.setup_prescaler(_56),
                        _ => self.setup_prescaler(_64),
                    }
                }

                #[inline]
                fn setup_prescaler(&self, prescaler: ClockSourcePrescaler) -> ClockSourcePrescaler {

                    match prescaler {
                        _1 | _2 | _3 | _4 | _5 | _6 | _7 | _8 => {
                            self.tim.tax_ctl.modify(|_,w| w.id().id_0());
                            self.tim.tax_ex0.modify(|_, w| unsafe {w.bits(prescaler as u16 -1)} );
                        },
                        _10 | _12 | _14 | _16 => {
                            self.tim.tax_ctl.modify(|_,w| w.id().id_1());
                            self.tim.tax_ex0.modify(|_, w| unsafe {w.bits(prescaler as u16 / 2 -1)} );
                        },
                        _20 | _24 | _28 | _32 => {
                            self.tim.tax_ctl.modify(|_,w| w.id().id_2());
                            self.tim.tax_ex0.modify(|_, w| unsafe {w.bits(prescaler as u16 / 4 -1)} );
                        },
                        _40 | _48 | _56 | _64 => {
                            self.tim.tax_ctl.modify(|_,w| w.id().id_3());
                            self.tim.tax_ex0.modify(|_, w| unsafe {w.bits(prescaler as u16 / 8 -1)} );
                        },
                    }

                    prescaler
                }

                #[inline]
                fn start_timer(&self) {
                    self.tim.tax_ctl.modify(|_, w| w.mc().mc_1());
                }
            }

            impl CountDown for TimerConfig <$TIMER, ClockDefined> {
                type Error = Error;
                type Time = Count;

                fn try_start <T>(&mut self, count: T) -> Result<(), Self::Error>
                where
                    T: Into<Self::Time> {
                    if(!self.timer_running()) {
                        self.clear_timer();

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

            impl Cancel for TimerConfig <$TIMER, ClockDefined> {
                fn try_cancel(&mut self) -> Result<(), Self::Error> {
                    if(self.timer_running()) {
                        self.stop_timer();
                        Ok(())
                    } else {
                        Err(Error::Disabled)
                    }
                }
            }

            impl Periodic for TimerConfig <$TIMER, ClockDefined> {}
        )*
    };
}

timer! {
    TIMER_A0, tim0,
    TIMER_A1, tim1,
    TIMER_A2, tim2,
    TIMER_A3, tim3
}

/// ********************************            TIMER 32            ********************************

use pac::TIMER32;
use core::marker::PhantomData;
use core::mem::transmute;

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
                       self.tim.$t32risi.read().raw_ifg().bits()) {
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

            impl Cancel for Timer32Config <$Channeli, ClockDefined> {
                fn try_cancel(&mut self) -> Result<(), Self::Error> {
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