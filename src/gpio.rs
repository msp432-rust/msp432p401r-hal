use core::marker::PhantomData;

pub use hal::digital::{InputPin, OutputPin, ToggleableOutputPin};
use pac::dio::*;

pub trait InputMode {}

pub trait OutputDrive {}

pub trait GPIO {
    type Parts;

    /// Split the port into all PINs and registers
    fn split(self) -> Self::Parts;
}

pub struct PulledUp;

pub struct PulledDown;

pub struct Floating;

pub struct RegularDrive;

pub struct HighDrive;

impl InputMode for PulledUp {}

impl InputMode for PulledDown {}

impl InputMode for Floating {}

impl OutputDrive for RegularDrive {}

impl OutputDrive for HighDrive {}

pub struct Input<T> where T: InputMode {
    _mode: PhantomData<T>
}

// TODO: Implement OutputDrive
pub struct Output;

macro_rules! gpio {
    ($portx:ident, $pxdir:ident, $pxout:ident, $pxin:ident, $pxren:ident, $pxds:ident, $PXx: ident, {
        $($PIx:ident: [
            $($PI_i:ident: ($pxi:ident, $i:expr, $pidir:ident, $piout:ident, $piin:ident, $piren:ident, $pids:ident, $MODE:ty),)+
        ])+
    }) => {
            pub mod $portx {
                use hal::digital::{OutputPin, InputPin, ToggleableOutputPin};
                use core::marker::PhantomData;
                use super::{Input, Output, InputMode, PulledUp, PulledDown, Floating, GPIO};
                use pac::DIO;
                use pac::Peripherals;

                /// Port Group Implementation (PA, PB, PC...)
                pub struct $PXx<MODE> {
                    i: u8,
                    _mode: PhantomData<MODE>,
                }

                /// Port Implementation (P1, P2, P3..)
                $(
                    pub struct $PIx<MODE> {
                        _mode: PhantomData<MODE>,
                    }

                    /// Pin Implementation (P1_1, P1_2...)
                    $(
                        pub struct $PI_i<MODE> {
                            i: u8,
                            _mode: PhantomData<MODE>,
                        }

                        impl<MODE> $PI_i<MODE> {
                            pub fn default() -> $PI_i<$MODE> {
                                $PI_i::<$MODE> {
                                    i: $i,
                                    _mode: PhantomData,
                                }
                            }

                            pub fn into_pulled_up_input(self) -> $PI_i<Input<PulledUp>> {
                                let dio = unsafe { &*DIO::ptr() };
                                dio.$pxdir.modify(|r,w| unsafe {
                                    w.$pidir().bits(r.$pidir().bits() & 0b0)
                                });
                                dio.$pxren.modify(|r,w| unsafe {
                                    w.$piren().bits(r.$piren().bits() | (0x01 << self.i))
                                });
                                dio.$pxout.modify(|r,w| unsafe {
                                    w.$piout().bits(r.$piout().bits() | (0x01 << self.i))
                                });
                                $PI_i::<Input<PulledUp>> { i: $i, _mode: PhantomData }
                            }

                            pub fn into_pulled_down_input(self) -> $PI_i<Input<PulledDown>> {
                                let dio = unsafe { &*DIO::ptr() };
                                dio.$pxdir.modify(|r,w| unsafe {
                                    w.$pidir().bits(r.$pidir().bits() & 0b0)
                                });
                                dio.$pxren.modify(|r,w| unsafe {
                                    w.$piren().bits(r.$piren().bits() | (0x01 << self.i))
                                });
                                dio.$pxout.modify(|r,w| unsafe {
                                    w.$piout().bits(r.$piout().bits() & !(0x01 << self.i))
                                });
                                $PI_i::<Input<PulledDown>> { i: $i, _mode: PhantomData }
                            }

                            pub fn into_floating_input(self) -> $PI_i<Input<Floating>> {
                                let dio = unsafe { &*DIO::ptr() };
                                dio.$pxdir.modify(|r,w| unsafe {
                                    w.$pidir().bits(r.$pidir().bits() & 0b0)
                                });
                                dio.$pxren.modify(|r,w| unsafe {
                                    w.$piren().bits(r.$piren().bits() & !(0x01 << self.i))
                                });
                                $PI_i::<Input<Floating>> { i: $i, _mode: PhantomData }
                            }

                            // TODO: Implement Drive Selection register
                            pub fn into_output(self) -> $PI_i<Output> {
                                let dio = unsafe { &*DIO::ptr() };
                                dio.$pxdir.modify(|r,w| unsafe {
                                    w.$pidir().bits(r.$pidir().bits() | 0b1)
                                });
                                $PI_i::<Output> { i: $i, _mode: PhantomData }
                            }
                        }

                        impl<M: InputMode> InputPin for $PI_i<Input<M>> {
                            type Error = ();

                            fn try_is_high(&self) -> Result<bool, Self::Error> {
                                let dio = unsafe { &*DIO::ptr() };
                                let state = (dio.$pxin.read().$piin().bits() & (0x01 << self.i)) > 0;
                                Ok(state)
                            }

                            fn try_is_low(&self) -> Result<bool, Self::Error> {
                                let dio = unsafe { &*DIO::ptr() };
                                let state = (!dio.$pxin.read().$piin().bits() & (0x01 << self.i)) > 0;
                                Ok(state)
                            }
                        }

                        impl OutputPin for $PI_i<Output> {
                            type Error = ();

                            fn try_set_low(&mut self) -> Result<(), Self::Error> {
                                let dio = unsafe { &*DIO::ptr() };
                                dio.$pxout.modify(|r,w| unsafe {
                                    w.$piout().bits(r.$piout().bits() & 0b0)
                                });
                                Ok(())
                            }

                            fn try_set_high(&mut self) -> Result<(), Self::Error> {
                                let dio = unsafe { &*DIO::ptr() };
                                dio.$pxout.modify(|r,w| unsafe {
                                    w.$piout().bits(r.$piout().bits() | 0b1)
                                });
                                Ok(())
                            }
                        }

                        impl ToggleableOutputPin for $PI_i<Output> {
                            type Error = ();

                            fn try_toggle(&mut self) -> Result<(), Self::Error> {
                                let dio = unsafe { &*DIO::ptr() };
                                dio.$pxout.modify(|r,w| unsafe {
                                    w.$piout().bits(r.$piout().bits() ^ 0b1)
                                });
                                Ok(())
                            }
                        }
                    )+
                )+

                pub struct Parts {
                    $(
                        $(
                            pub $pxi: $PI_i<$MODE>,
                        )+
                    )+
                }
            }
    }
}

gpio!(porta, padir, paout, pain, paren, pads, PAx, {
    P1x: [
        P1_0: (p1_0, 0, p1dir, p1out, p1in, p1ren, p1ds, Input<Floating>),
        P1_1: (p1_1, 1, p1dir, p1out, p1in, p1ren, p1ds, Input<Floating>),
        P1_2: (p1_2, 2, p1dir, p1out, p1in, p1ren, p1ds, Input<Floating>),
        P1_3: (p1_3, 3, p1dir, p1out, p1in, p1ren, p1ds, Input<Floating>),
        P1_4: (p1_4, 4, p1dir, p1out, p1in, p1ren, p1ds, Input<Floating>),
        P1_5: (p1_5, 5, p1dir, p1out, p1in, p1ren, p1ds, Input<Floating>),
        P1_6: (p1_6, 6, p1dir, p1out, p1in, p1ren, p1ds, Input<Floating>),
        P1_7: (p1_7, 7, p1dir, p1out, p1in, p1ren, p1ds, Input<Floating>),
    ]
    P2x: [
        P2_0: (p2_0, 0, p2dir, p2out, p2in, p2ren, p2ds, Input<Floating>),
        P2_1: (p2_1, 1, p2dir, p2out, p2in, p2ren, p2ds, Input<Floating>),
        P2_2: (p2_2, 2, p2dir, p2out, p2in, p2ren, p2ds, Input<Floating>),
        P2_3: (p2_3, 3, p2dir, p2out, p2in, p2ren, p2ds, Input<Floating>),
        P2_4: (p2_4, 4, p2dir, p2out, p2in, p2ren, p2ds, Input<Floating>),
        P2_5: (p2_5, 5, p2dir, p2out, p2in, p2ren, p2ds, Input<Floating>),
        P2_6: (p2_6, 6, p2dir, p2out, p2in, p2ren, p2ds, Input<Floating>),
        P2_7: (p2_7, 7, p2dir, p2out, p2in, p2ren, p2ds, Input<Floating>),
    ]
});

gpio!(portb, pbdir, pbout, pbin, pbren, pbds, PBx, {
    P3x: [
        P3_0: (p3_0, 0, p3dir, p3out, p3in, p3ren, p3ds, Input<Floating>),
        P3_1: (p3_1, 1, p3dir, p3out, p3in, p3ren, p3ds, Input<Floating>),
        P3_2: (p3_2, 2, p3dir, p3out, p3in, p3ren, p3ds, Input<Floating>),
        P3_3: (p3_3, 3, p3dir, p3out, p3in, p3ren, p3ds, Input<Floating>),
        P3_4: (p3_4, 4, p3dir, p3out, p3in, p3ren, p3ds, Input<Floating>),
        P3_5: (p3_5, 5, p3dir, p3out, p3in, p3ren, p3ds, Input<Floating>),
        P3_6: (p3_6, 6, p3dir, p3out, p3in, p3ren, p3ds, Input<Floating>),
        P3_7: (p3_7, 7, p3dir, p3out, p3in, p3ren, p3ds, Input<Floating>),
    ]
    P4x: [
        P4_0: (p4_0, 0, p4dir, p4out, p4in, p4ren, p4ds, Input<Floating>),
        P4_1: (p4_1, 1, p4dir, p4out, p4in, p4ren, p4ds, Input<Floating>),
        P4_2: (p4_2, 2, p4dir, p4out, p4in, p4ren, p4ds, Input<Floating>),
        P4_3: (p4_3, 3, p4dir, p4out, p4in, p4ren, p4ds, Input<Floating>),
        P4_4: (p4_4, 4, p4dir, p4out, p4in, p4ren, p4ds, Input<Floating>),
        P4_5: (p4_5, 5, p4dir, p4out, p4in, p4ren, p4ds, Input<Floating>),
        P4_6: (p4_6, 6, p4dir, p4out, p4in, p4ren, p4ds, Input<Floating>),
        P4_7: (p4_7, 7, p4dir, p4out, p4in, p4ren, p4ds, Input<Floating>),
    ]
});

gpio!(portc, pcdir, pcout, pcin, pcren, pcds, PCx, {
    P5x: [
        P5_0: (p5_0, 0, p5dir, p5out, p5in, p5ren, p5ds, Input<Floating>),
        P5_1: (p5_1, 1, p5dir, p5out, p5in, p5ren, p5ds, Input<Floating>),
        P5_2: (p5_2, 2, p5dir, p5out, p5in, p5ren, p5ds, Input<Floating>),
        P5_3: (p5_3, 3, p5dir, p5out, p5in, p5ren, p5ds, Input<Floating>),
        P5_4: (p5_4, 4, p5dir, p5out, p5in, p5ren, p5ds, Input<Floating>),
        P5_5: (p5_5, 5, p5dir, p5out, p5in, p5ren, p5ds, Input<Floating>),
        P5_6: (p5_6, 6, p5dir, p5out, p5in, p5ren, p5ds, Input<Floating>),
        P5_7: (p5_7, 7, p5dir, p5out, p5in, p5ren, p5ds, Input<Floating>),
    ]
    P6x: [
        P6_0: (p6_0, 0, p6dir, p6out, p6in, p6ren, p6ds, Input<Floating>),
        P6_1: (p6_1, 1, p6dir, p6out, p6in, p6ren, p6ds, Input<Floating>),
        P6_2: (p6_2, 2, p6dir, p6out, p6in, p6ren, p6ds, Input<Floating>),
        P6_3: (p6_3, 3, p6dir, p6out, p6in, p6ren, p6ds, Input<Floating>),
        P6_4: (p6_4, 4, p6dir, p6out, p6in, p6ren, p6ds, Input<Floating>),
        P6_5: (p6_5, 5, p6dir, p6out, p6in, p6ren, p6ds, Input<Floating>),
        P6_6: (p6_6, 6, p6dir, p6out, p6in, p6ren, p6ds, Input<Floating>),
        P6_7: (p6_7, 7, p6dir, p6out, p6in, p6ren, p6ds, Input<Floating>),
    ]
});

gpio!(portd, pddir, pdout, pdin, pdren, pdds, PDx, {
    P7x: [
        P7_0: (p7_0, 0, p7dir, p7out, p7in, p7ren, p7ds, Input<Floating>),
        P7_1: (p7_1, 1, p7dir, p7out, p7in, p7ren, p7ds, Input<Floating>),
        P7_2: (p7_2, 2, p7dir, p7out, p7in, p7ren, p7ds, Input<Floating>),
        P7_3: (p7_3, 3, p7dir, p7out, p7in, p7ren, p7ds, Input<Floating>),
        P7_4: (p7_4, 4, p7dir, p7out, p7in, p7ren, p7ds, Input<Floating>),
        P7_5: (p7_5, 5, p7dir, p7out, p7in, p7ren, p7ds, Input<Floating>),
        P7_6: (p7_6, 6, p7dir, p7out, p7in, p7ren, p7ds, Input<Floating>),
        P7_7: (p7_7, 7, p7dir, p7out, p7in, p7ren, p7ds, Input<Floating>),
    ]
    P8x: [
        P8_0: (p8_0, 0, p8dir, p8out, p8in, p8ren, p8ds, Input<Floating>),
        P8_1: (p8_1, 1, p8dir, p8out, p8in, p8ren, p8ds, Input<Floating>),
        P8_2: (p8_2, 2, p8dir, p8out, p8in, p8ren, p8ds, Input<Floating>),
        P8_3: (p8_3, 3, p8dir, p8out, p8in, p8ren, p8ds, Input<Floating>),
        P8_4: (p8_4, 4, p8dir, p8out, p8in, p8ren, p8ds, Input<Floating>),
        P8_5: (p8_5, 5, p8dir, p8out, p8in, p8ren, p8ds, Input<Floating>),
        P8_6: (p8_6, 6, p8dir, p8out, p8in, p8ren, p8ds, Input<Floating>),
        P8_7: (p8_7, 7, p8dir, p8out, p8in, p8ren, p8ds, Input<Floating>),
    ]
});

gpio!(porte, pedir, peout, pein, peren, peds, PEx, {
    P9x: [
        P9_0: (p9_0, 0, p9dir, p9out, p9in, p9ren, p9ds, Input<Floating>),
        P9_1: (p9_1, 1, p9dir, p9out, p9in, p9ren, p9ds, Input<Floating>),
        P9_2: (p9_2, 2, p9dir, p9out, p9in, p9ren, p9ds, Input<Floating>),
        P9_3: (p9_3, 3, p9dir, p9out, p9in, p9ren, p9ds, Input<Floating>),
        P9_4: (p9_4, 4, p9dir, p9out, p9in, p9ren, p9ds, Input<Floating>),
        P9_5: (p9_5, 5, p9dir, p9out, p9in, p9ren, p9ds, Input<Floating>),
        P9_6: (p9_6, 6, p9dir, p9out, p9in, p9ren, p9ds, Input<Floating>),
        P9_7: (p9_7, 7, p9dir, p9out, p9in, p9ren, p9ds, Input<Floating>),
    ]
    P10x: [
        P10_0: (p10_0, 0, p10dir, p10out, p10in, p10ren, p10ds, Input<Floating>),
        P10_1: (p10_1, 1, p10dir, p10out, p10in, p10ren, p10ds, Input<Floating>),
        P10_2: (p10_2, 2, p10dir, p10out, p10in, p10ren, p10ds, Input<Floating>),
        P10_3: (p10_3, 3, p10dir, p10out, p10in, p10ren, p10ds, Input<Floating>),
        P10_4: (p10_4, 4, p10dir, p10out, p10in, p10ren, p10ds, Input<Floating>),
        P10_5: (p10_5, 5, p10dir, p10out, p10in, p10ren, p10ds, Input<Floating>),
        P10_6: (p10_6, 6, p10dir, p10out, p10in, p10ren, p10ds, Input<Floating>),
        P10_7: (p10_7, 7, p10dir, p10out, p10in, p10ren, p10ds, Input<Floating>),
    ]
});
