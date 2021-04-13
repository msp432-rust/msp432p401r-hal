use hal::spi::*;
use pac::{EUSCI_A0, EUSCI_A1, EUSCI_A2, EUSCI_A3};
use pac::{EUSCI_B0, EUSCI_B1, EUSCI_B2, EUSCI_B3};
use pac::eusci_a0;

use crate::gpio::{Alternate, Primary};
use crate::gpio::porta::*;
use crate::gpio::portb::*;
use crate::gpio::portc::*;
use crate::gpio::portd::*;
use crate::gpio::porte::*;

use super::SPI;

pub enum ClockSource {
    ACLK,
    SMCLK,
}

pub struct Disabled;

pub struct Enabled;

macro_rules! spi_a {
    ($($SPI_Xi:ident: [$EUSCI:ident, $STE:ty, $CLK:ty, $MISO:ty, $MOSI:ty],)+) => {
        $(
            pub struct $SPI_Xi<State> {
                eusci: $EUSCI,
                _state: State,
            }

            impl<State> $SPI_Xi<State> {
                fn new(eusci: $EUSCI) -> $SPI_Xi<Disabled> {
                    eusci.ucax_ctlw0.modify(|_, w| { w
                        .ucswrst().ucswrst_1()
                    });
                    $SPI_Xi { eusci: eusci, _state: Disabled }
                }
            }

            impl $SPI_Xi<Enabled> {
                pub fn disable(self) -> $SPI_Xi<Disabled> {
                    $SPI_Xi::<Disabled>::new(self.eusci)
                }
                // TODO: return ownership of Pins to the caller
                pub fn release(self) {

                }
            }

            impl $SPI_Xi<Disabled> {
                pub fn with_clock_source(self, source: ClockSource) -> Self {
                    match source {
                        ClockSource::ACLK => self.eusci.ucax_ctlw0.modify(|_, w| w.ucssel().ucssel_1()),
                        ClockSource::SMCLK => self.eusci.ucax_ctlw0.modify(|_, w| w.ucssel().ucssel_2()),
                    }
                    self
                }

                pub fn with_mode(self, mode: Mode) -> Self {
                    match mode {
                        Mode::MODE_0 =>  self.eusci.ucax_ctlw0.modify(|r,w| unsafe { w
                            .bits(r.bits() | (0x01 << 15))
                            .bits(r.bits() & !(0x01 << 14))
                        }),
                        Mode::MODE_1 =>  self.eusci.ucax_ctlw0.modify(|r,w| unsafe { w
                            .bits(r.bits() & !(0x01 << 15))
                            .bits(r.bits() & !(0x01 << 14))
                        }),
                        Mode::MODE_2 =>  self.eusci.ucax_ctlw0.modify(|r,w| unsafe { w
                            .bits(r.bits() | (0x01 << 15))
                            .bits(r.bits() | (0x01 << 14))
                        }),
                        Mode::MODE_3 =>  self.eusci.ucax_ctlw0.modify(|r,w| unsafe { w
                            .bits(r.bits() & !(0x01 << 15))
                            .bits(r.bits() | (0x01 << 14))
                        }),
                    }
                    self
                }

                pub fn msb_first(self) -> Self {
                    self.eusci.ucax_ctlw0.modify(|r,w| unsafe { w.bits(r.bits() | (0x01 << 13)) });
                    self
                }

                pub fn lsb_first(self) -> Self {
                    self.eusci.ucax_ctlw0.modify(|r,w| unsafe { w.bits(r.bits() & !(0x01 << 13)) });
                    self
                }

                pub fn master_mode(self) -> Self {
                    self.eusci.ucax_ctlw0.modify(|r, w| unsafe { w.bits(r.bits() | (0x01 << 11)) });
                    self
                }

                pub fn slave_mode(self) -> Self {
                    self.eusci.ucax_ctlw0.modify(|r, w| unsafe { w.bits(r.bits() & !(0x01 << 11)) });
                    self
                }

                // TODO: Should we keep a reference to the pins?
                pub fn with_ports(self, ste: $STE, clk: $CLK, miso: $MISO, mosi: $MOSI) -> Self {
                    self
                }

                pub fn init(self) -> $SPI_Xi<Enabled> {
                    self.eusci.ucax_ctlw0.modify(|_, w| { w
                        .ucsync().ucsync_1()
                        .ucswrst().ucswrst_0()
                    });
                    $SPI_Xi { eusci: self.eusci, _state: Enabled }
                }
            }

            // TODO: This!
            impl FullDuplex for $SPI_Xi<Enabled> {

            }

            impl SPI for $EUSCI {
                type Module = $SPI_Xi<Disabled>;

                fn into_spi(self) -> $SPI_Xi<Disabled> {
                    $SPI_Xi::<Disabled>::new(self)
                    // Set UCSWRST
                    // Initialize all eUSCI registers with UCSWRST = 1 (including UCxCTL1).
                    // Configure ports.
                    // Clear UCSWRST
                }
            }
        )+
    }
}

spi_a! {               // STE                    // CLK                    // MISO                   // MOSI
    SPI_A0: [EUSCI_A0, P1_0<Alternate<Primary>>, P1_1<Alternate<Primary>>, P1_2<Alternate<Primary>>, P1_3<Alternate<Primary>>],
    SPI_A1: [EUSCI_A1, P2_0<Alternate<Primary>>, P2_1<Alternate<Primary>>, P2_2<Alternate<Primary>>, P2_3<Alternate<Primary>>],
    SPI_A2: [EUSCI_A2, P3_0<Alternate<Primary>>, P3_1<Alternate<Primary>>, P3_2<Alternate<Primary>>, P3_3<Alternate<Primary>>],
    SPI_A3: [EUSCI_A3, P9_4<Alternate<Primary>>, P9_5<Alternate<Primary>>, P9_6<Alternate<Primary>>, P9_7<Alternate<Primary>>],

    // SPI_B0: [EUSCI_B0, P1_4<Alternate<Primary>>, P1_5<Alternate<Primary>>, P1_7<Alternate<Primary>>, P1_6<Alternate<Primary>>],
    // SPI_B1: [EUSCI_B1, P6_2<Alternate<Primary>>, P6_3<Alternate<Primary>>, P6_5<Alternate<Primary>>, P6_4<Alternate<Primary>>],
    // SPI_B2: [EUSCI_B2, P3_4<Alternate<Primary>>, P3_5<Alternate<Primary>>, P3_7<Alternate<Primary>>, P3_6<Alternate<Primary>>],
    // SPI_B3: [EUSCI_B3, P10_0<Alternate<Primary>>, P10_1<Alternate<Primary>>, P10_3<Alternate<Primary>>, P10_2<Alternate<Primary>>],
}
