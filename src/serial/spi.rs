use pac::{EUSCI_A0, EUSCI_A1, EUSCI_A2, EUSCI_A3};
use pac::{EUSCI_B0, EUSCI_B1, EUSCI_B2, EUSCI_B3};

pub use hal::spi::{MODE_0, MODE_1, MODE_2, MODE_3};

use super::SPI;

pub enum ClockSource {
    ACLK,
    SMCLK,
}

pub enum Error {
    Unknown,
}

pub struct SpiBuilder<EUSCI> {
    eusci: EUSCI
}

macro_rules! spi {
    (
        $(($spix:ident,$ucx_ctlw0:ident, $ucx_brw:ident, $ucx_statw:ident, $ucx_rx:ident, $ucx_tx:ident, $ucx_ie:ident, $ucx_ifg:ident, $ucx_iv:ident): {
            $($SPI_Xi:ident: $EUSCI:ident,)+
        })+
    ) => {
        $(
            pub mod $spix {
                use super::*;
                use hal::spi::*;

                $(
                    pub struct $SPI_Xi {
                        eusci: $EUSCI,
                    }

                    impl $SPI_Xi {
                        pub fn disable(self) -> SpiBuilder<$EUSCI> {
                            SpiBuilder::<$EUSCI>::new(self.eusci)
                        }

                        // TODO: Implement embedded-hal traits for SPI (blocking and non-blocking)
                        pub fn write(&self, data: u8) {
                            self.eusci.$ucx_tx.write(|w| unsafe { w.bits(data as u16) });
                        }

                        pub fn read(&self) -> u8 {
                            self.eusci.$ucx_rx.read().bits() as u8
                        }
                    }

                    impl SpiBuilder<$EUSCI> {
                        fn new(eusci: $EUSCI) -> SpiBuilder<$EUSCI> {
                            eusci.$ucx_ctlw0.modify(|_, w| { w.ucswrst().ucswrst_1() });
                            SpiBuilder { eusci: eusci }
                        }

                        pub fn with_clock_source(self, source: ClockSource) -> Self {
                            match source {
                                ClockSource::ACLK => self.eusci.$ucx_ctlw0.modify(|_, w| w.ucssel().ucssel_1()),
                                ClockSource::SMCLK => self.eusci.$ucx_ctlw0.modify(|_, w| w.ucssel().ucssel_2()),
                            }
                            self
                        }

                        pub fn with_mode(self, mode: Mode) -> Self {
                            match mode {
                                MODE_0 =>  self.eusci.$ucx_ctlw0.modify(|r,w| unsafe { w
                                    .bits(r.bits() | (0x01 << 15))
                                    .bits(r.bits() & !(0x01 << 14))
                                }),
                                MODE_1 =>  self.eusci.$ucx_ctlw0.modify(|r,w| unsafe { w
                                    .bits(r.bits() & !(0x01 << 15))
                                    .bits(r.bits() & !(0x01 << 14))
                                }),
                                MODE_2 =>  self.eusci.$ucx_ctlw0.modify(|r,w| unsafe { w
                                    .bits(r.bits() | (0x01 << 15))
                                    .bits(r.bits() | (0x01 << 14))
                                }),
                                MODE_3 =>  self.eusci.$ucx_ctlw0.modify(|r,w| unsafe { w
                                    .bits(r.bits() & !(0x01 << 15))
                                    .bits(r.bits() | (0x01 << 14))
                                }),
                            }
                            self
                        }

                        pub fn msb_first(self) -> Self {
                            self.eusci.$ucx_ctlw0.modify(|r,w| unsafe { w.bits(r.bits() | (0x01 << 13)) });
                            self
                        }

                        pub fn lsb_first(self) -> Self {
                            self.eusci.$ucx_ctlw0.modify(|r,w| unsafe { w.bits(r.bits() & !(0x01 << 13)) });
                            self
                        }

                        pub fn master_mode(self) -> Self {
                            self.eusci.$ucx_ctlw0.modify(|r, w| unsafe { w.bits(r.bits() | (0x01 << 11)) });
                            self
                        }

                        pub fn slave_mode(self) -> Self {
                            self.eusci.$ucx_ctlw0.modify(|r, w| unsafe { w.bits(r.bits() & !(0x01 << 11)) });
                            self
                        }

                        pub fn with_bit_rate_prescaler(self, prescaler: u16) -> Self {
                            self.eusci.$ucx_brw.modify(|_,w| unsafe { w.bits(prescaler) });
                            self
                        }

                        pub fn init(self) -> $SPI_Xi {
                            self.eusci.$ucx_ctlw0.modify(|_, w| { w
                                .ucsync().ucsync_1()
                                .ucswrst().ucswrst_0()
                            });
                            $SPI_Xi { eusci: self.eusci }
                        }
                    }

                    impl SPI for $EUSCI {
                        type Module = SpiBuilder<$EUSCI>;

                        fn into_spi(self) -> SpiBuilder<$EUSCI> {
                            SpiBuilder::<$EUSCI>::new(self)
                        }
                    }
                )+
            }
        )+
    }
}

spi! {
    (spia, ucax_ctlw0, ucax_brw, ucax_statw, ucax_rxbuf, ucax_txbuf, ucax_ie, ucax_ifg, ucax_iv): {
        SPI_A0: EUSCI_A0,
        SPI_A1: EUSCI_A1,
        SPI_A2: EUSCI_A2,
        SPI_A3: EUSCI_A3,
    }
    (spib, ucbx_ctlw0, ucbx_brw, ucbx_statw, ucbx_rxbuf, ucbx_txbuf, ucbx_ie, ucbx_ifg, ucbx_iv): {
        SPI_B0: EUSCI_B0,
        SPI_B1: EUSCI_B1,
        SPI_B2: EUSCI_B2,
        SPI_B3: EUSCI_B3,
    }
}
