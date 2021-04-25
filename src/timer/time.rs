//! Time Units
/// Frequency unit - Bits per second
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Bps(pub u32);
/// Frequency unit - Hertz
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Hertz(pub u32);
/// Frequency unit - KiloHertz
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct KiloHertz(pub u32);
/// Frequency unit - MegaHertz
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct MegaHertz(pub u32);
/// Time unit - MilliSeconds
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct MilliSeconds(pub u32);
/// Time unit - Seconds
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Seconds(pub u32);
/// Enum of Timer Units
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum TimerUnits {
    Bps          = 0x00,
    Hertz        = 0x01,
    Kilohertz    = 0x02,
    Megahertz    = 0x03,
    Milliseconds = 0x04,
    Seconds      = 0x05,
}
/// Struct of Count
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct TimeCount {
    pub value: u32,
    pub unity: TimerUnits,
}
/// Extension trait that adds convenience methods to the `u32` type
pub trait TimerUnit {
    /// Wrap in `Bps`
    fn bps(self) -> Bps;
    /// Wrap in `Hertz`
    fn hertz(self) -> Hertz;
    /// Wrap in `KiloHertz`
    fn kilohertz(self) -> KiloHertz;
    /// Wrap in `MegaHertz`
    fn megahertz(self) -> MegaHertz;
    /// Wrap in `MilliSeconds`
    fn milliseconds(self) -> MilliSeconds;
    /// Wrap in `Seconds`
    fn seconds(self) -> Seconds;
}
/// Impl TimerUnit for u32
impl TimerUnit for u32 {
    fn bps(self) -> Bps {
        Bps(self)
    }
    fn hertz(self) -> Hertz {
        Hertz(self)
    }
    fn kilohertz(self) -> KiloHertz {
        KiloHertz(self)
    }
    fn megahertz(self) -> MegaHertz {
        MegaHertz(self)
    }
    fn milliseconds(self) -> MilliSeconds {
        MilliSeconds(self)
    }
    fn seconds(self) -> Seconds {
        Seconds(self)
    }
}
/// Impl From Bps -> u32
impl From<Bps> for u32 {
    fn from(t: Bps) -> Self {
        t.0
    }
}
/// Impl From Hertz -> u32
impl From<Hertz> for u32 {
    fn from(t: Hertz) -> Self {
        t.0
    }
}
/// Impl From KiloHertz -> u32
impl From<KiloHertz> for u32 {
    fn from(t: KiloHertz) -> Self {
        t.0
    }
}
/// Impl From MegaHertz -> u32
impl From<MegaHertz> for u32 {
    fn from(t: MegaHertz) -> Self {
        t.0
    }
}
/// Impl From MilliSeconds -> u32
impl From<MilliSeconds> for u32 {
    fn from(t: MilliSeconds) -> Self {
        t.0
    }
}
/// Impl From Seconds -> u32
impl From<Seconds> for u32 {
    fn from(t: Seconds) -> Self {
        t.0
    }
}
/// Impl From u32 -> Bps
impl From<u32> for Bps {
    fn from(t: u32) -> Self {
        t.bps()
    }
}
/// Impl From u32 -> Hertz
impl From<u32> for Hertz {
    fn from(t: u32) -> Self {
        t.hertz()
    }
}
/// Impl From u32 -> KiloHertz
impl From<u32> for KiloHertz {
    fn from(t: u32) -> Self {
        t.kilohertz()
    }
}
/// Impl From u32 -> MegaHertz
impl From<u32> for MegaHertz {
    fn from(t: u32) -> Self {
        t.megahertz()
    }
}
/// Impl From u32 -> MilliSeconds
impl From<u32> for MilliSeconds {
    fn from(t: u32) -> Self {
        t.milliseconds()
    }
}
/// Impl From u32 -> Seconds
impl From<u32> for Seconds {
    fn from(t: u32) -> Self {
        t.seconds()
    }
}
/// Impl Into Seconds -> MilliSeconds
impl Into<MilliSeconds> for Seconds {
    fn into(self) -> MilliSeconds {
        MilliSeconds(self.0 * 1_000)
    }
}
/// Impl Into MilliSeconds -> Seconds
impl Into<Seconds> for MilliSeconds {
    fn into(self) -> Seconds {
        Seconds(self.0 / 1_000)
    }
}
/// Impl Into Bps -> Hertz
impl Into<Hertz> for Bps {
    fn into(self) -> Hertz {
        Hertz(self.0)
    }
}
/// Impl Into Hertz -> Bps
impl Into<Bps> for Hertz {
    fn into(self) -> Bps {
        Bps(self.0)
    }
}
/// Impl Into KiloHertz -> Hertz
impl Into<Hertz> for KiloHertz {
    fn into(self) -> Hertz {
        Hertz(self.0 * 1_000)
    }
}
/// Impl Into Hertz -> KiloHertz
impl Into<KiloHertz> for Hertz {
    fn into(self) -> KiloHertz {
        KiloHertz(self.0 / 1_000)
    }
}
/// Impl Into MegaHertz -> Hertz
impl Into<Hertz> for MegaHertz {
    fn into(self) -> Hertz {
        Hertz(self.0 * 1_000_000)
    }
}
/// Impl Into Hertz -> MegaHertz
impl Into<MegaHertz> for Hertz {
    fn into(self) -> MegaHertz {
        MegaHertz(self.0 / 1_000_000)
    }
}
/// Impl Into MegaHertz -> KiloHertz
impl Into<KiloHertz> for MegaHertz {
    fn into(self) -> KiloHertz {
        KiloHertz(self.0 * 1_000)
    }
}
/// Impl Into KiloHertz -> MegaHertz
impl Into<MegaHertz> for KiloHertz {
    fn into(self) -> MegaHertz {
        MegaHertz(self.0 / 1_000)
    }
}
/// Impl From Bps -> TimeCount
impl From<Bps> for TimeCount {
    fn from(t: Bps) -> Self {
        TimeCount {
            value: u32::from(t),
            unity: TimerUnits::Bps,
        }
    }
}
/// Impl From Hertz -> TimeCount
impl From<Hertz> for TimeCount {
    fn from(t: Hertz) -> Self {
        TimeCount {
            value: u32::from(t),
            unity: TimerUnits::Hertz,
        }
    }
}
/// Impl From KiloHertz -> TimeCount
impl From<KiloHertz> for TimeCount {
    fn from(t: KiloHertz) -> Self {
        TimeCount {
            value: u32::from(t),
            unity: TimerUnits::Kilohertz,
        }
    }
}
/// Impl From MegaHertz -> TimeCount
impl From<MegaHertz> for TimeCount {
    fn from(t: MegaHertz) -> Self {
        TimeCount {
            value: u32::from(t),
            unity: TimerUnits::Megahertz,
        }
    }
}
/// Impl From MilliSeconds -> TimeCount
impl From<MilliSeconds> for TimeCount {
    fn from(t: MilliSeconds) -> Self {
        TimeCount {
            value: u32::from(t),
            unity: TimerUnits::Milliseconds,
        }
    }
}
/// Impl From Seconds -> TimeCount
impl From<Seconds> for TimeCount {
    fn from(t: Seconds) -> Self {
        TimeCount {
            value: u32::from(t),
            unity: TimerUnits::Seconds,
        }
    }
}
/// Impl From TimeCount -> Bps
impl From<TimeCount> for Bps {
    fn from(t: TimeCount) -> Self {
        match t.unity {
            TimerUnits::Bps => Bps(t.value),
            TimerUnits::Hertz => Bps(t.value),
            TimerUnits::Kilohertz => Bps(t.value * 1000),
            TimerUnits::Megahertz => Bps(t.value * 1_000_000),
            TimerUnits::Milliseconds => Bps(1000 / t.value),
            _=> Bps(0),
        }
    }
}
/// Impl From TimeCount -> Hertz
impl From<TimeCount> for Hertz {
    fn from(t: TimeCount) -> Self {
        match t.unity {
            TimerUnits::Bps => Hertz(t.value),
            TimerUnits::Hertz => Hertz(t.value),
            TimerUnits::Kilohertz => Hertz(t.value * 1000),
            TimerUnits::Megahertz => Hertz(t.value * 1_000_000),
            TimerUnits::Milliseconds => Hertz(1000 / t.value),
            _=> Hertz(0),
        }
    }
}
/// Impl From TimeCount -> KiloHertz
impl From<TimeCount> for KiloHertz {
    fn from(t: TimeCount) -> Self {
        match t.unity {
            TimerUnits::Bps => KiloHertz(t.value / 1000),
            TimerUnits::Hertz => KiloHertz(t.value / 1000),
            TimerUnits::Kilohertz => KiloHertz(t.value),
            TimerUnits::Megahertz => KiloHertz(t.value * 1000),
            _=> KiloHertz(0),
        }
    }
}
/// Impl From TimeCount -> MegaHertz
impl From<TimeCount> for MegaHertz {
    fn from(t: TimeCount) -> Self {
        match t.unity {
            TimerUnits::Bps => MegaHertz(t.value / 1_000_000),
            TimerUnits::Hertz => MegaHertz(t.value / 1_000_000),
            TimerUnits::Kilohertz => MegaHertz(t.value / 1000),
            TimerUnits::Megahertz => MegaHertz(t.value),
            _=> MegaHertz(0),
        }
    }
}
/// Impl From TimeCount -> MilliSeconds
impl From<TimeCount> for MilliSeconds {
    fn from(t: TimeCount) -> Self {
        match t.unity {
            TimerUnits::Milliseconds => MilliSeconds(t.value),
            TimerUnits::Seconds => MilliSeconds(t.value * 1000),
            TimerUnits::Hertz => MilliSeconds(1000 / t.value),
            TimerUnits::Bps => MilliSeconds(1000 / t.value),
            _=> MilliSeconds(0),
        }
    }
}
/// Impl From TimeCount -> Seconds
impl From<TimeCount> for Seconds {
    fn from(t: TimeCount) -> Self {
        match t.unity {
            TimerUnits::Milliseconds => Seconds(t.value / 1000),
            TimerUnits::Seconds => Seconds(t.value),
            _=> Seconds(0),
        }
    }
}
/// Impl TimerUnits
impl TimeCount {
    /// Check Period
    pub fn is_period(&self) -> bool {
        match self.unity {
            TimerUnits::Milliseconds | TimerUnits::Seconds  => true,
            _=> false,
        }
    }
    /// Check Frequency
    pub fn is_frequency(&self) -> bool {
        match self.unity {
            TimerUnits::Milliseconds | TimerUnits::Seconds  => false,
            _=> true,
        }
    }
}
