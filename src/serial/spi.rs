use core::marker::PhantomData;

use hal::spi::FullDuplex;
use pac::{EUSCI_A0, EUSCI_A1, EUSCI_A2, EUSCI_A3};
use pac::{EUSCI_B0, EUSCI_B1, EUSCI_B2, EUSCI_B3};

use crate::gpio::Parts;
use crate::gpio::porta::*;

use super::eusci::{Serial, SerialSPI, SPI};

macro_rules! spi {
    ($($EUSCI:ident: [$($STE:ident, $CLK:ident, $SOMI:ident, $SIMO:ident)*],)+) => {
        $(
            impl SerialSPI for $EUSCI {
                fn into_spi(self, gpio: Parts) -> Serial<SPI> {
                    Serial { _mode: <SPI>::_new() }
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
    EUSCI_A0: [P1_0, P1_1, P1_2, P1_3],
    EUSCI_A1: [P2_0, P2_1, P2_2, P2_3],
    EUSCI_A2: [P3_0, P3_1, P3_2, P3_3],
    EUSCI_A3: [P4_0, P4_1, P4_2, P4_3],
}
