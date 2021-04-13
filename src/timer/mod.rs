pub mod timer16;
pub mod timer32;

pub use timer32::*;
pub use timer16::*;

pub trait State {}
pub struct ClockNotDefined;
pub struct ClockDefined;

impl State for ClockNotDefined {}
impl State for ClockDefined {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    Disabled,
    Enabled,
    Unreachable,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum TimerUnit {
    Hertz,
    Kilohertz,
    Milliseconds,
    Seconds,
}

#[derive(PartialEq, PartialOrd)]
pub struct Count(pub u32, pub TimerUnit);