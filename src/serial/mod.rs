#[allow(non_camel_case_types)]
pub mod spi;
pub mod uart;
pub mod i2c;

pub trait SPI {
    type Module;
    fn into_spi(self) -> Self::Module;
}

pub trait I2C {
    type Module;
    fn into_i2c(self) -> Self::Module;
}

pub trait UART {
    type Module;
    fn into_uart(self) -> Self::Module;
}
