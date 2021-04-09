use core::marker::PhantomData;

use hal::spi::FullDuplex;
use pac::{EUSCI_A0, EUSCI_A1, EUSCI_A2, EUSCI_A3};
use pac::{EUSCI_B0, EUSCI_B1, EUSCI_B2, EUSCI_B3};

use crate::gpio::{Parts, Alternate, Primary};
use crate::gpio::porta::*;

use super::eusci::SPI;

macro_rules! spi {
    ($($SPI_Xi:ident: [$EUSCI:ident, $STE:ty, $CLK:ty, $SOMI:ty, $SIMO:ty],)+) => {
        $(
            pub struct $SPI_Xi {
                /// eUSCI SPI Clock
                clk: $CLK,
                /// Slave Transmit Enable
                ste: $STE,
                /// Slave out / Master in
                somi: $SOMI,
                /// Slave in / Master out
                simo: $SIMO
            }

            /// Setup I/O ports into relevant alternate modes
            impl $SPI_Xi {
                pub fn setup_ports(self, ste: $STE, clk: $CLK, somi: $SOMI, simo: $SIMO) -> Self {
                    Self { clk, ste, somi, simo }
                }
            }

            impl SPI for $EUSCI {
                type Module = $SPI_Xi;

                fn into_spi(self) -> $SPI_Xi {
                    $SPI_Xi {}

                    // Set UCSWRST
                    // Initialize all eUSCI registers with UCSWRST = 1 (including UCxCTL1).
                    // Configure ports.
                    // Clear UCSWRST

                }
            }
        )+
    }
}

spi! {
    SPI_A0: [EUSCI_A0, P1_0<Alternate<Primary>>, P1_1<Alternate<Primary>>, P1_2<Alternate<Primary>>, P1_3<Alternate<Primary>>],
}
