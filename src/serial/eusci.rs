use core::marker::PhantomData;

use pac::{EUSCI_A0, EUSCI_A1, EUSCI_A2, EUSCI_A3};
use pac::{EUSCI_B0, EUSCI_B1, EUSCI_B2, EUSCI_B3};
use crate::gpio::Parts;

pub struct SPI;
pub struct I2C;
pub struct UART;

pub trait SerialSPI {
    fn into_spi(self, gpio: Parts) -> Serial<SPI>;
}

pub trait SerialI2C {
    fn into_i2c(self, gpio: Parts) -> Serial<I2C>;
}

pub trait SerialUART {
    fn into_uart(self, gpio: Parts) -> Serial<UART>;
}

pub struct Serial<MODE> {
    _mode: PhantomData<MODE>
}

impl<MODE> Serial<MODE> {
    const fn _new() -> Self { Self { _mode: PhantomData } }
}

// macro_rules! spi {
//     ($($EUSCI:ident: [$($STE:ident, $CLK:ident, $SOMI:ident, $SIMO:ident)*],)+) => {
//         $(
//             impl SPI for $EUSCI {
//                 fn setup(self) {
//                     // Set UCSWRST
//                     // Initialize all eUSCI registers with UCSWRST = 1 (including UCxCTL1).
//                     // Configure ports.
//                     // Clear UCSWRST
//
//
//
//                     todo!()
//                 }
//             }
//         )+
//     }
// }
