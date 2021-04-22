pub mod timer16;
pub mod timer32;

pub use timer32::*;
pub use timer16::*;

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