#[allow(non_camel_case_types)]
pub mod spi;
pub mod uart;
pub mod i2c;

pub trait SPI {
    type Module;
    fn into_spi(self) -> Self::Module;
}

pub trait I2C {
    fn into_i2c(self) -> Self;
}

pub trait UART {
    fn into_uart(self) -> Self;
}
