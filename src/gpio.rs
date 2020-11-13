use core::marker::PhantomData;

use hal::digital::{InputPin, OutputPin, ToggleableOutputPin};
use pac::DIO;

pub trait InputMode {}

pub trait OutputDrive {}

pub struct PulledUp;

pub struct PulledDown;

pub struct RegularDrive;

pub struct HighDrive;

impl InputMode for PulledUp {}

impl InputMode for PulledDown {}

impl OutputDrive for RegularDrive {}

impl OutputDrive for HighDrive {}

pub struct Input<T> where T: InputMode {
    _mode: PhantomData<T>
}

pub struct Output;

macro_rules! gpio {
    ($portx:ident, $PXx: ident, [
        $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty),)+
        ]) => {
            pub mod $portx {
                use hal::digital::{OutputPin, InputPin, ToggleableOutputPin};
                use core::marker::PhantomData;
                use super::{Input, Output, InputMode, PulledUp, PulledDown};
                use pac::Peripherals;

                /// Port Implementation
                pub struct $PXx<MODE> {
                    i: u8,
                    _mode: PhantomData<MODE>,
                }

                // TODO: Is this really necessary?
                impl OutputPin for $PXx<Output> {
                    type Error = ();

                    fn try_set_low(&mut self) -> Result<(), Self::Error> {
                        unimplemented!()
                    }

                    fn try_set_high(&mut self) -> Result<(), Self::Error> {
                        unimplemented!()
                    }
                }

                /// PIN Implementation
                $(
                    pub struct $PXi<MODE> {
                        _mode: PhantomData<MODE>,
                    }

                    impl<MODE> $PXi<MODE> {
                        pub fn into_pulled_up_input(&self) -> $PXi<Input<PulledUp>> {
                            unimplemented!()
                        }

                        pub fn into_pulled_down_input(&self) -> $PXi<Input<PulledDown>> {
                            unimplemented!()
                        }

                        pub fn into_output(&self) -> $PXi<Output> {
                            unimplemented!()
                        }
                    }

                    impl<M: InputMode> InputPin for $PXi<Input<M>> {
                        type Error = ();

                        fn try_is_high(&self) -> Result<bool, Self::Error> {
                            unimplemented!()
                        }

                        fn try_is_low(&self) -> Result<bool, Self::Error> {
                            unimplemented!()
                        }
                    }

                    impl OutputPin for $PXi<Output> {
                        type Error = ();

                        fn try_set_low(&mut self) -> Result<(), Self::Error> {
                            unimplemented!()
                        }

                        fn try_set_high(&mut self) -> Result<(), Self::Error> {
                            unimplemented!()
                        }
                    }

                    impl ToggleableOutputPin for $PXi<Output> {
                        type Error = ();

                        fn try_toggle(&mut self) -> Result<(), Self::Error> {
                            DIO::ptr().
                        }
                    }
                )+
            }
    }
}

gpio!(porta, PAx, [
    PA0: (pa0, 0, Input<PulledUp>),
    PA1: (pa1, 1, Input<PulledUp>),
    PA2: (pa2, 2, Input<PulledUp>),
    PA3: (pa3, 3, Input<PulledUp>),
    PA4: (pa4, 4, Input<PulledUp>),
    PA5: (pa5, 5, Input<PulledUp>),
    PA6: (pa6, 6, Input<PulledUp>),
    PA7: (pa7, 7, Input<PulledUp>),
]);
