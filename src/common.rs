//! Hal Common Resources - MSP432P401R

pub struct NotDefined;
pub struct Defined;

pub trait Constrain<T> {
    fn constrain(self) -> T;
}

pub trait Split {
    type Parts;
    fn split(self) -> Self::Parts;
}