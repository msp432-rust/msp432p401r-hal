use core::marker::PhantomData;

use pac::{EUSCI_A0, EUSCI_A1, EUSCI_A2, EUSCI_A3};
use pac::{EUSCI_B0, EUSCI_B1, EUSCI_B2, EUSCI_B3};
use crate::gpio::Parts;

pub trait SPI {
    type Module;
    type CLK;
    type MISO;
    type MOSI;
    type STE;

    fn into_spi(self, clk: Self::CLK, miso: Self::MISO, mosi: Self::MOSI, ste: Self::STE) -> Self::Module;
}

pub trait I2C {
    fn into_i2c(self) -> Self;
}

pub trait UART {
    fn into_uart(self) -> Self;
}
