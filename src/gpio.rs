use core::marker::PhantomData;

pub use hal::digital::{InputPin, OutputPin, ToggleableOutputPin};

pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

impl<MODE> Input<MODE> {
    const fn _new() -> Self {
        Self { _mode: PhantomData }
    }
}

pub struct Floating;
pub struct PullDown;
pub struct PullUp;

pub struct Output {
    _mode: PhantomData<()>,
}

impl Output {
    const fn _new() -> Self {
        Self { _mode: PhantomData }
    }
}

pub struct Alternate<MODE> {
    _mode: PhantomData<MODE>,
}

impl<MODE> Alternate<MODE> {
    const fn _new() -> Self {
        Self { _mode: PhantomData }
    }
}

pub struct Primary;
pub struct Secondary;
pub struct Tertiary;

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(self) -> Self::Parts;
}

macro_rules! gpio {
    ($DIO:ident, {
        $(($portx:ident, $pxdir:ident, $pxout:ident, $pxin:ident, $pxren:ident, $pxsel0:ident, $pxsel1:ident, $PXx:ident): {
          $($PIx:ident: [
            $($PI_i:ident: ($pxi:ident, $i:expr, $j:expr, $pidir:ident, $piout:ident, $piin:ident, $piren:ident, $pisel0:ident, $pisel1:ident, $MODE:ty),)+
          ])+
        })+
    }) => {
        $(
            pub mod $portx {
                use hal::digital::{OutputPin, InputPin, ToggleableOutputPin};
                use core::convert::Infallible;
                use core::marker::PhantomData;
                use super::*;

                /// Port Group Implementation (PA, PB, PC...)
                pub struct $PXx<MODE> {
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
                            pub(crate) _mode: MODE,
                        }

                        impl<MODE> $PI_i<MODE> {
                            /// Setup PullUp resistor and configures Pin to Input mode
                            pub fn into_pull_up_input(self) -> $PI_i<Input<PullUp>> {
                                let dio = unsafe { &*$DIO::ptr() };
                                dio.$pxdir.modify(|r,w| unsafe {
                                    w.$pidir().bits(r.$pidir().bits() & !(0x01 << $i))
                                });
                                dio.$pxren.modify(|r,w| unsafe {
                                    w.$piren().bits(r.$piren().bits() | (0x01 << $i))
                                });
                                dio.$pxout.modify(|r,w| unsafe {
                                    w.$piout().bits(r.$piout().bits() | (0x01 << $i))
                                });
                                $PI_i { _mode: Input::_new() }
                            }

                            /// Setup PullDown resistor and configures Pin to Input mode
                            pub fn into_pull_down_input(self) -> $PI_i<Input<PullDown>> {
                                let dio = unsafe { &*$DIO::ptr() };
                                dio.$pxdir.modify(|r,w| unsafe {
                                    w.$pidir().bits(r.$pidir().bits() & !(0x01 << $i))
                                });
                                dio.$pxren.modify(|r,w| unsafe {
                                    w.$piren().bits(r.$piren().bits() | (0x01 << $i))
                                });
                                dio.$pxout.modify(|r,w| unsafe {
                                    w.$piout().bits(r.$piout().bits() & !(0x01 << $i))
                                });
                                $PI_i { _mode: Input::_new() }
                            }

                            /// Disables PullUp/PullDown resistor and configures Pin to Input mode
                            pub fn into_floating_input(self) -> $PI_i<Input<Floating>> {
                                let dio = unsafe { &*$DIO::ptr() };
                                dio.$pxdir.modify(|r,w| unsafe {
                                    w.$pidir().bits(r.$pidir().bits() & !(0x01 << $i))
                                });
                                dio.$pxren.modify(|r,w| unsafe {
                                    w.$piren().bits(r.$piren().bits() & !(0x01 << $i))
                                });
                                $PI_i { _mode: Input::_new() }
                            }

                            // TODO: Implement Drive Selection register
                            /// Setup Pin to output mode
                            pub fn into_output(self) -> $PI_i<Output> {
                                let dio = unsafe { &*$DIO::ptr() };
                                dio.$pxdir.modify(|r,w| unsafe {
                                    w.$pidir().bits(r.$pidir().bits() | (0x01 << $i))
                                });
                                $PI_i { _mode: Output::_new() }
                            }

                            /// Setup Primary Module Function
                            pub fn into_alternate_primary(self) -> $PI_i<Alternate<Primary>> {
                                let dio = unsafe { &*$DIO::ptr() };
                                dio.$pxsel1.modify(|r,w| unsafe {
                                    w.$pisel1().bits(r.$pisel1().bits() & !(0x01 << $i))
                                });
                                dio.$pxsel0.modify(|r,w| unsafe {
                                    w.$pisel0().bits(r.$pisel0().bits() | (0x01 << $i))
                                });
                                $PI_i { _mode: Alternate::_new() }
                            }

                            /// Setup Secondary Module Function
                            pub fn into_alternate_secondary(self) -> $PI_i<Alternate<Secondary>> {
                                let dio = unsafe { &*$DIO::ptr() };
                                dio.$pxsel1.modify(|r,w| unsafe {
                                    w.$pisel1().bits(r.$pisel1().bits() | (0x01 << $i))
                                });
                                dio.$pxsel0.modify(|r,w| unsafe {
                                    w.$pisel0().bits(r.$pisel0().bits() & !(0x01 << $i))
                                });
                                $PI_i { _mode: Alternate::_new() }
                            }

                            /// Setup Tertiary Module Function
                            pub fn into_alternate_tertiary(self) -> $PI_i<Alternate<Tertiary>> {
                                let dio = unsafe { &*$DIO::ptr() };
                                dio.$pxsel1.modify(|r,w| unsafe {
                                    w.$pisel1().bits(r.$pisel1().bits() | (0x01 << $i))
                                });
                                dio.$pxsel0.modify(|r,w| unsafe {
                                    w.$pisel0().bits(r.$pisel0().bits() | (0x01 << $i))
                                });
                                $PI_i { _mode: Alternate::_new() }
                            }
                        }

                        impl<MODE> InputPin for $PI_i<Input<MODE>> {
                            type Error = Infallible;

                            fn try_is_high(&self) -> Result<bool, Self::Error> {
                                self.try_is_low().map(|low| !low)
                            }

                            fn try_is_low(&self) -> Result<bool, Self::Error> {
                                let dio = unsafe { &*$DIO::ptr() };
                                let state = (dio.$pxin.read().$piin().bits() & (0x01 << $i)) == 0;
                                Ok(state)
                            }
                        }

                        impl OutputPin for $PI_i<Output> {
                            type Error = Infallible;

                            fn try_set_low(&mut self) -> Result<(), Self::Error> {
                                let dio = unsafe { &*$DIO::ptr() };
                                dio.$pxout.modify(|r,w| unsafe {
                                    w.$piout().bits(r.$piout().bits() & !(0x01 << $i))
                                });
                                Ok(())
                            }

                            fn try_set_high(&mut self) -> Result<(), Self::Error> {
                                let dio = unsafe { &*$DIO::ptr() };
                                dio.$pxout.modify(|r,w| unsafe {
                                    w.$piout().bits(r.$piout().bits() | (0x01 << $i))
                                });
                                Ok(())
                            }
                        }

                        impl ToggleableOutputPin for $PI_i<Output> {
                            type Error = Infallible;

                            fn try_toggle(&mut self) -> Result<(), Self::Error> {
                                let dio = unsafe { &*$DIO::ptr() };
                                    dio.$pxout.modify(|r,w| unsafe {
                                    w.$piout().bits(r.$piout().bits() ^ (0x01 << $i))
                                });
                                Ok(())
                            }
                        }
                    )+
                )+
            }
        )+

        pub struct Parts {
            $($($(
                pub $pxi: $portx::$PI_i<$MODE>,
            )+)+)+
        }

        impl GpioExt for $DIO {
            type Parts = Parts;

            fn split(self) -> Parts {
                Parts {
                    $($($(
                        $pxi: $portx::$PI_i { _mode: <$MODE>::_new() },
                    )+)+)+
                }
            }
        }
    }
}

use pac::DIO;

gpio!(DIO, {
    (porta, padir, paout, pain, paren, pasel0, pasel1, PAx): {
        P1x: [
            P1_0: (p1_0, 0, 0, p1dir, p1out, p1in, p1ren, p1sel0, p1sel1, Input<Floating>),
            P1_1: (p1_1, 1, 1, p1dir, p1out, p1in, p1ren, p1sel0, p1sel1, Input<Floating>),
            P1_2: (p1_2, 2, 2, p1dir, p1out, p1in, p1ren, p1sel0, p1sel1, Input<Floating>),
            P1_3: (p1_3, 3, 3, p1dir, p1out, p1in, p1ren, p1sel0, p1sel1, Input<Floating>),
            P1_4: (p1_4, 4, 4, p1dir, p1out, p1in, p1ren, p1sel0, p1sel1, Input<Floating>),
            P1_5: (p1_5, 5, 5, p1dir, p1out, p1in, p1ren, p1sel0, p1sel1, Input<Floating>),
            P1_6: (p1_6, 6, 6, p1dir, p1out, p1in, p1ren, p1sel0, p1sel1, Input<Floating>),
            P1_7: (p1_7, 7, 7, p1dir, p1out, p1in, p1ren, p1sel0, p1sel1, Input<Floating>),
        ]
        P2x: [
            P2_0: (p2_0, 0,  8, p2dir, p2out, p2in, p2ren, p2sel0, p2sel1, Input<Floating>),
            P2_1: (p2_1, 1,  9, p2dir, p2out, p2in, p2ren, p2sel0, p2sel1, Input<Floating>),
            P2_2: (p2_2, 2, 10, p2dir, p2out, p2in, p2ren, p2sel0, p2sel1, Input<Floating>),
            P2_3: (p2_3, 3, 11, p2dir, p2out, p2in, p2ren, p2sel0, p2sel1, Input<Floating>),
            P2_4: (p2_4, 4, 12, p2dir, p2out, p2in, p2ren, p2sel0, p2sel1, Input<Floating>),
            P2_5: (p2_5, 5, 13, p2dir, p2out, p2in, p2ren, p2sel0, p2sel1, Input<Floating>),
            P2_6: (p2_6, 6, 14, p2dir, p2out, p2in, p2ren, p2sel0, p2sel1, Input<Floating>),
            P2_7: (p2_7, 7, 15, p2dir, p2out, p2in, p2ren, p2sel0, p2sel1, Input<Floating>),
        ]
    }
    (portb, pbdir, pbout, pbin, pbren, pbsel0, pbsel1, PBx): {
        P3x: [
            P3_0: (p3_0, 0, 0, p3dir, p3out, p3in, p3ren, p3sel0, p3sel1, Input<Floating>),
            P3_1: (p3_1, 1, 1, p3dir, p3out, p3in, p3ren, p3sel0, p3sel1, Input<Floating>),
            P3_2: (p3_2, 2, 2, p3dir, p3out, p3in, p3ren, p3sel0, p3sel1, Input<Floating>),
            P3_3: (p3_3, 3, 3, p3dir, p3out, p3in, p3ren, p3sel0, p3sel1, Input<Floating>),
            P3_4: (p3_4, 4, 4, p3dir, p3out, p3in, p3ren, p3sel0, p3sel1, Input<Floating>),
            P3_5: (p3_5, 5, 5, p3dir, p3out, p3in, p3ren, p3sel0, p3sel1, Input<Floating>),
            P3_6: (p3_6, 6, 6, p3dir, p3out, p3in, p3ren, p3sel0, p3sel1, Input<Floating>),
            P3_7: (p3_7, 7, 7, p3dir, p3out, p3in, p3ren, p3sel0, p3sel1, Input<Floating>),
        ]
        P4x: [
            P4_0: (p4_0, 0,  8, p4dir, p4out, p4in, p4ren, p4sel0, p4sel1, Input<Floating>),
            P4_1: (p4_1, 1,  9, p4dir, p4out, p4in, p4ren, p4sel0, p4sel1, Input<Floating>),
            P4_2: (p4_2, 2, 10, p4dir, p4out, p4in, p4ren, p4sel0, p4sel1, Input<Floating>),
            P4_3: (p4_3, 3, 11, p4dir, p4out, p4in, p4ren, p4sel0, p4sel1, Input<Floating>),
            P4_4: (p4_4, 4, 12, p4dir, p4out, p4in, p4ren, p4sel0, p4sel1, Input<Floating>),
            P4_5: (p4_5, 5, 13, p4dir, p4out, p4in, p4ren, p4sel0, p4sel1, Input<Floating>),
            P4_6: (p4_6, 6, 14, p4dir, p4out, p4in, p4ren, p4sel0, p4sel1, Input<Floating>),
            P4_7: (p4_7, 7, 15, p4dir, p4out, p4in, p4ren, p4sel0, p4sel1, Input<Floating>),
        ]
    }
    (portc, pcdir, pcout, pcin, pcren, pcsel0, pcsel1, PCx): {
        P5x: [
            P5_0: (p5_0, 0, 0, p5dir, p5out, p5in, p5ren, p5sel0, p5sel1, Input<Floating>),
            P5_1: (p5_1, 1, 1, p5dir, p5out, p5in, p5ren, p5sel0, p5sel1, Input<Floating>),
            P5_2: (p5_2, 2, 2, p5dir, p5out, p5in, p5ren, p5sel0, p5sel1, Input<Floating>),
            P5_3: (p5_3, 3, 3, p5dir, p5out, p5in, p5ren, p5sel0, p5sel1, Input<Floating>),
            P5_4: (p5_4, 4, 4, p5dir, p5out, p5in, p5ren, p5sel0, p5sel1, Input<Floating>),
            P5_5: (p5_5, 5, 5, p5dir, p5out, p5in, p5ren, p5sel0, p5sel1, Input<Floating>),
            P5_6: (p5_6, 6, 6, p5dir, p5out, p5in, p5ren, p5sel0, p5sel1, Input<Floating>),
            P5_7: (p5_7, 7, 7, p5dir, p5out, p5in, p5ren, p5sel0, p5sel1, Input<Floating>),
        ]
        P6x: [
            P6_0: (p6_0, 0,  8, p6dir, p6out, p6in, p6ren, p6sel0, p6sel1, Input<Floating>),
            P6_1: (p6_1, 1,  9, p6dir, p6out, p6in, p6ren, p6sel0, p6sel1, Input<Floating>),
            P6_2: (p6_2, 2, 10, p6dir, p6out, p6in, p6ren, p6sel0, p6sel1, Input<Floating>),
            P6_3: (p6_3, 3, 11, p6dir, p6out, p6in, p6ren, p6sel0, p6sel1, Input<Floating>),
            P6_4: (p6_4, 4, 12, p6dir, p6out, p6in, p6ren, p6sel0, p6sel1, Input<Floating>),
            P6_5: (p6_5, 5, 13, p6dir, p6out, p6in, p6ren, p6sel0, p6sel1, Input<Floating>),
            P6_6: (p6_6, 6, 14, p6dir, p6out, p6in, p6ren, p6sel0, p6sel1, Input<Floating>),
            P6_7: (p6_7, 7, 15, p6dir, p6out, p6in, p6ren, p6sel0, p6sel1, Input<Floating>),
        ]
    }
    (portd, pddir, pdout, pdin, pdren, pdsel0, pdsel1, PDx): {
        P7x: [
            P7_0: (p7_0, 0, 0, p7dir, p7out, p7in, p7ren, p7sel0, p7sel1, Input<Floating>),
            P7_1: (p7_1, 1, 1, p7dir, p7out, p7in, p7ren, p7sel0, p7sel1, Input<Floating>),
            P7_2: (p7_2, 2, 2, p7dir, p7out, p7in, p7ren, p7sel0, p7sel1, Input<Floating>),
            P7_3: (p7_3, 3, 3, p7dir, p7out, p7in, p7ren, p7sel0, p7sel1, Input<Floating>),
            P7_4: (p7_4, 4, 4, p7dir, p7out, p7in, p7ren, p7sel0, p7sel1, Input<Floating>),
            P7_5: (p7_5, 5, 5, p7dir, p7out, p7in, p7ren, p7sel0, p7sel1, Input<Floating>),
            P7_6: (p7_6, 6, 6, p7dir, p7out, p7in, p7ren, p7sel0, p7sel1, Input<Floating>),
            P7_7: (p7_7, 7, 7, p7dir, p7out, p7in, p7ren, p7sel0, p7sel1, Input<Floating>),
        ]
        P8x: [
            P8_0: (p8_0, 0,  8, p8dir, p8out, p8in, p8ren, p8sel0, p8sel1, Input<Floating>),
            P8_1: (p8_1, 1,  9, p8dir, p8out, p8in, p8ren, p8sel0, p8sel1, Input<Floating>),
            P8_2: (p8_2, 2, 10, p8dir, p8out, p8in, p8ren, p8sel0, p8sel1, Input<Floating>),
            P8_3: (p8_3, 3, 11, p8dir, p8out, p8in, p8ren, p8sel0, p8sel1, Input<Floating>),
            P8_4: (p8_4, 4, 12, p8dir, p8out, p8in, p8ren, p8sel0, p8sel1, Input<Floating>),
            P8_5: (p8_5, 5, 13, p8dir, p8out, p8in, p8ren, p8sel0, p8sel1, Input<Floating>),
            P8_6: (p8_6, 6, 14, p8dir, p8out, p8in, p8ren, p8sel0, p8sel1, Input<Floating>),
            P8_7: (p8_7, 7, 15, p8dir, p8out, p8in, p8ren, p8sel0, p8sel1, Input<Floating>),
        ]
    }
    (porte, pedir, peout, pein, peren, pesel0, pesel1, PEx): {
        P9x: [
            P9_0: (p9_0, 0, 0, p9dir, p9out, p9in, p9ren, p9sel0, p9sel1, Input<Floating>),
            P9_1: (p9_1, 1, 1, p9dir, p9out, p9in, p9ren, p9sel0, p9sel1, Input<Floating>),
            P9_2: (p9_2, 2, 2, p9dir, p9out, p9in, p9ren, p9sel0, p9sel1, Input<Floating>),
            P9_3: (p9_3, 3, 3, p9dir, p9out, p9in, p9ren, p9sel0, p9sel1, Input<Floating>),
            P9_4: (p9_4, 4, 4, p9dir, p9out, p9in, p9ren, p9sel0, p9sel1, Input<Floating>),
            P9_5: (p9_5, 5, 5, p9dir, p9out, p9in, p9ren, p9sel0, p9sel1, Input<Floating>),
            P9_6: (p9_6, 6, 6, p9dir, p9out, p9in, p9ren, p9sel0, p9sel1, Input<Floating>),
            P9_7: (p9_7, 7, 7, p9dir, p9out, p9in, p9ren, p9sel0, p9sel1, Input<Floating>),
        ]
        P10x: [
            P10_0: (p10_0, 0,  8, p10dir, p10out, p10in, p10ren, p10sel0, p10sel1, Input<Floating>),
            P10_1: (p10_1, 1,  9, p10dir, p10out, p10in, p10ren, p10sel0, p10sel1, Input<Floating>),
            P10_2: (p10_2, 2, 10, p10dir, p10out, p10in, p10ren, p10sel0, p10sel1, Input<Floating>),
            P10_3: (p10_3, 3, 11, p10dir, p10out, p10in, p10ren, p10sel0, p10sel1, Input<Floating>),
            P10_4: (p10_4, 4, 12, p10dir, p10out, p10in, p10ren, p10sel0, p10sel1, Input<Floating>),
            P10_5: (p10_5, 5, 13, p10dir, p10out, p10in, p10ren, p10sel0, p10sel1, Input<Floating>),
            P10_6: (p10_6, 6, 14, p10dir, p10out, p10in, p10ren, p10sel0, p10sel1, Input<Floating>),
            P10_7: (p10_7, 7, 15, p10dir, p10out, p10in, p10ren, p10sel0, p10sel1, Input<Floating>),
        ]
    }
});
