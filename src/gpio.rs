use pac::{DIO};
use pac::Peripherals;
use hal::digital::v2::{OutputPin, InputPin, ToggleableOutputPin};

pub enum Mode {
    PullUp,
    PullDown,
}

pub struct Pin {
    mode: Mode,
}

pub struct PA {
    p1: Pin,
    p2: Pin,
    p3: Pin,
    p4: Pin,
    p5: Pin,
}

impl OutputPin for Pin {
    type Error = ();

    fn set_low(&mut self) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

impl InputPin for Pin {
    type Error = ();

    fn is_high(&self) -> Result<bool, Self::Error> {
        unimplemented!()
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        unimplemented!()
    }
}

impl ToggleableOutputPin for Pin {
    type Error = ();

    fn toggle(&mut self) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
