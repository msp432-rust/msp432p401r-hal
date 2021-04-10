use core::marker::PhantomData;

use hal::spi::FullDuplex;
use pac::{EUSCI_A0, EUSCI_A1, EUSCI_A2, EUSCI_A3};
use pac::{EUSCI_B0, EUSCI_B1, EUSCI_B2, EUSCI_B3};

use crate::gpio::{Parts, Alternate, Primary};
use crate::gpio::porta::*;

use super::eusci::SPI;

macro_rules! spi {
    ($($SPI_Xi:ident: [$EUSCI:ident, $STE:ty, $CLK:ty, $MISO:ty, $MOSI:ty],)+) => {
        $(
            pub struct $SPI_Xi {
                /// eUSCI SPI Clock
                clk: $CLK,
                /// Slave Transmit Enable
                ste: $STE,
                /// Master in / Slave out
                MISO: $MISO,
                /// Master out / Slave in
                MOSI: $MOSI
            }

            impl $SPI_Xi {
                fn new(clk: $CLK, MISO: $MISO, MOSI: $MOSI, ste: $STE) -> Self {
                    Self { clk, ste, MISO, MOSI }
                }

                pub fn set_clock() {

                }

                pub fn enable() {

                }

                pub fn disable() {

                }
            }

            impl SPI for $EUSCI {
                type Module = $SPI_Xi;
                type MOSI = $MOSI;
                type MISO = $MISO;
                type STE = $STE;
                type CLK = $CLK;

                fn into_spi(self, ste: $STE, clk: $CLK, miso: $MISO, mosi: $MOSI) -> $SPI_Xi {
                    $SPI_Xi::new(clk, miso, mosi, ste)

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
