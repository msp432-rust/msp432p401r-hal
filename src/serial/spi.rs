use core::marker::PhantomData;

use hal::spi::FullDuplex;
use pac::{EUSCI_A0, EUSCI_A1, EUSCI_A2, EUSCI_A3};
use pac::{EUSCI_B0, EUSCI_B1, EUSCI_B2, EUSCI_B3};

use crate::gpio::{Parts, Alternate, Primary, Input, Floating, State};
use crate::gpio::porta::*;

trait SPI_A {
    type STE;
    type CLK;
    type MISO;
    type MOSI;

    fn ste(self) -> Self::STE;
    fn clk(self) -> Self::CLK;
    fn miso(self) -> Self::MISO;
    fn mosi(self) -> Self::MOSI;
}

pub struct Disabled;
pub struct Enabled;

impl Enabled {

}

use super::SPI;

macro_rules! spi_a {
    ($($SPI_Xi:ident: [$EUSCI:ident, $STE:ty, $CLK:ty, $MISO:ty, $MOSI:ty],)+) => {
        $(
            pub struct $SPI_Xi<State> {
                _state: State

                // /// eUSCI SPI Clock
                // clk: $CLK,
                // /// Slave Transmit Enable
                // ste: $STE,
                // /// Master in / Slave out
                // miso: $MISO,
                // /// Master out / Slave in
                // mosi: $MOSI
            }

            impl SPI_A for $SPI_Xi<Enabled> {
                type STE = $STE;
                type CLK = $CLK;
                type MISO = $MISO;
                type MOSI = $MOSI;
            }

            impl $SPI_Xi<Enabled> {
                pub fn release() {

                }
            }

            impl $SPI_Xi<Disabled> {
                fn new() -> $SPI_Xi<Disabled> {
                    $EUSCI::ptr().ucax_ctlw0.modify(|_, w| w.ucswrst().ucswrst_1());
                    $SPI_Xi { _state: PhantomData::<Disabled>() }
                }

                pub fn set_clock() -> Self {

                }

                pub fn setup_ports(ste: $STE, clk: $CLK, miso: $MISO, mosi: $MOSI) -> Self {

                }

                pub fn init() -> $SPI_Xi<Enabled> {

                }
            }

            impl SPI for $EUSCI {
                type Module = $SPI_Xi;

                fn into_spi(self) -> $SPI_Xi<Disabled> {
                    $SPI_Xi::new()
                    // Set UCSWRST
                    // Initialize all eUSCI registers with UCSWRST = 1 (including UCxCTL1).
                    // Configure ports.
                    // Clear UCSWRST

                }
            }
        )+
    }
}



fn banana() {
    // EUSCI_A0::ptr().ucax_ctlw0.modify(|_, w| w.ucswrst().ucswrst_1());
}

spi_a! {
    SPI_A0: [EUSCI_A0, P1_0<Alternate<Primary>>, P1_1<Alternate<Primary>>, P1_2<Alternate<Primary>>, P1_3<Alternate<Primary>>],
    // SPI_A1: [EUSCI_A1, P1_0<Alternate<Primary>>, P1_1<Alternate<Primary>>, P1_2<Alternate<Primary>>, P1_3<Alternate<Primary>>],
    // SPI_A2: [EUSCI_A2, P1_0<Alternate<Primary>>, P1_1<Alternate<Primary>>, P1_2<Alternate<Primary>>, P1_3<Alternate<Primary>>],
    // SPI_A3: [EUSCI_A3, P1_0<Alternate<Primary>>, P1_1<Alternate<Primary>>, P1_2<Alternate<Primary>>, P1_3<Alternate<Primary>>],
}
