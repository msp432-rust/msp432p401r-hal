use pac::{EUSCI_B0, EUSCI_B1, EUSCI_B2, EUSCI_B3};

use super::I2C;

pub enum ClockSource {
    UCLK,
    ACLK,
    SMCLK,
}

pub enum AddressMode {
    _7bits,
    _10bits,
}

pub struct I2cBuilder<EUSCI> {
    eusci: EUSCI
}

macro_rules! i2c {
    ($($I2C_Xi:ident: $EUSCI:ident,)+) => {
        $(
            pub struct $I2C_Xi {
                eusci: $EUSCI
            }

            impl I2cBuilder<$EUSCI> {
                fn new(eusci: $EUSCI) -> Self {
                    eusci.ucbx_ctlw0.modify(|_, w| w.ucswrst().ucswrst_1().ucmode().ucmode_3());
                    I2cBuilder { eusci }
                }

                pub fn own_addressing_mode(self, mode: AddressMode) -> Self {
                    match mode {
                        AddressMode::_7bits => self.eusci.ucbx_ctlw0.modify(|_, w| w.uca10().uca10_0()),
                        AddressMode::_10bits => self.eusci.ucbx_ctlw0.modify(|_, w| w.uca10().uca10_1()),
                    }
                    self
                }

                pub fn slave_addressing_mode(self, mode: AddressMode) -> Self {
                    match mode {
                        AddressMode::_7bits => self.eusci.ucbx_ctlw0.modify(|_, w| w.ucsla10().ucsla10_0()),
                        AddressMode::_10bits => self.eusci.ucbx_ctlw0.modify(|_, w| w.ucsla10().ucsla10_1()),
                    }
                    self
                }

                pub fn slave_mode(self) -> Self {
                    self.eusci.ucbx_ctlw0.modify(|_, w| w.ucmst().ucmst_0());
                    self
                }

                pub fn master_mode(self) -> Self {
                    self.eusci.ucbx_ctlw0.modify(|_, w| w.ucmst().ucmst_1());
                    self
                }

                pub fn multi_master(self) -> Self {
                    self.eusci.ucbx_ctlw0.modify(|_, w| w.ucmm().ucmm_1());
                    self
                }

                pub fn single_master(self) -> Self {
                    self.eusci.ucbx_ctlw0.modify(|_, w| w.ucmm().ucmm_0());
                    self
                }

                pub fn with_clock_source(self, source: ClockSource) -> Self {
                    match source {
                        ClockSource::UCLK => self.eusci.ucbx_ctlw0.modify(|_, w| w.ucssel().ucssel_0()),
                        ClockSource::ACLK => self.eusci.ucbx_ctlw0.modify(|_, w| w.ucssel().ucssel_1()),
                        ClockSource::SMCLK => self.eusci.ucbx_ctlw0.modify(|_, w| w.ucssel().ucssel_2()),
                    }
                    self
                }

                pub fn with_bit_rate_prescaler(self, prescaler: u16) -> Self {
                    self.eusci.ucbx_brw.modify(|_, w| unsafe { w.bits(prescaler) });
                    self
                }

                /// Only evaluates bits 0-9 in 10-bit address mode or 0-6 in 7-bit address modes
                pub fn with_own_address(self, address: u16) -> Self {
                    let address_mask: u16 = 0xFC00;
                    self.eusci.ucbx_i2coa0.modify(|r, w| unsafe { w.bits( r.bits() | (address & address_mask) ) });
                    self
                }

                pub fn init(self) -> $I2C_Xi {
                    self.eusci.ucbx_ctlw0.modify(|_, w| w.ucswrst().ucswrst_0());
                    $I2C_Xi { eusci: self.eusci }
                }
            }

            impl $I2C_Xi {
                pub fn disable(self) -> I2cBuilder<$EUSCI> {
                    I2cBuilder::<$EUSCI>::new(self.eusci)
                }
            }

            impl I2C for $EUSCI {
                type Module = I2cBuilder<$EUSCI>;

                fn into_i2c(self) -> I2cBuilder<$EUSCI> {
                    I2cBuilder::<$EUSCI>::new(self)
                }
            }
        )+
    }
}

i2c! {
    I2C_B0: EUSCI_B0,
    I2C_B1: EUSCI_B1,
    I2C_B2: EUSCI_B2,
    I2C_B3: EUSCI_B3,
}
