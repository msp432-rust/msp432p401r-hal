use pac::{DIO};
use pac::Peripherals;
use hal::digital::{InputPin, OutputPin, ToggleableOutputPin};

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

    fn try_set_low(&mut self) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn try_set_high(&mut self) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

impl InputPin for Pin {
    type Error = ();

    fn try_is_high(&self) -> Result<bool, Self::Error> {
        unimplemented!()
    }

    fn try_is_low(&self) -> Result<bool, Self::Error> {
        unimplemented!()
    }
}

impl ToggleableOutputPin for Pin {
    type Error = ();

    fn try_toggle(&mut self) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
