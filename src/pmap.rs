const UNLOCK_KEY: u16 = 0x2D52;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Mapping {
  NONE     =  0,
  UCA0CLK  =  1,
  UCA0SOMI =  2,
  UCA0TXD  =  3,
  UCB0CLK  =  4,
  UCB0SDA  =  5,
  UCB0SCL  =  6,
  UCA1STE  =  7,
  UCA1CLK  =  8,
  UCA1RXD  =  9,
  UCA1TXD  = 10,
  UCA2STE  = 11,
  UCA2CLK  = 12,
  UCA2RXD  = 13,
  UCA2TXD  = 14,
  UCB2STE  = 15,
  UCB2CLK  = 16,
  UCB2SDA  = 17,
  UCB2SCL  = 18,
  TA0CCR0A = 19,
  TA0CCR1A = 20,
  TA0CCR2A = 21,
  TA0CCR3A = 22,
  TA0CCR4A = 23,
  TA1CCR1A = 24,
  TA1CCR2A = 25,
  TA1CCR3A = 26,
  TA1CCR4A = 27,
  TA0CLK   = 28,
  TA1CLK   = 29,
  DMAE0    = 30,
  ANALOG   = 31,
}

impl Mapping {
  pub const UCA0RXD: Self  =  Self::UCA0SOMI;
  pub const UCA0SIMO: Self  = Self::UCA0TXD;
  pub const UCB0SIMO: Self  = Self::UCB0SDA;
  pub const UCB0SOMI: Self  = Self::UCB0SCL;
  pub const UCA1SOMI: Self  = Self::UCA1RXD;
  pub const UCA1SIMO: Self  = Self::UCA1TXD;
  pub const UCA2SOMI: Self  = Self::UCA2RXD;
  pub const UCA2SIMO: Self  = Self::UCA2TXD;
  pub const UCB2SIMO: Self  = Self::UCB2SDA;
  pub const UCB2SOMI: Self  = Self::UCB2SCL;
  pub const CE0OUT: Self    = Self::TA0CLK;
  pub const CE1OUT: Self    = Self::TA1CLK;
  pub const SMCLK: Self     = Self::DMAE0;
}

pub trait PmapExt {
  unsafe fn configure<T>(&mut self, allow_reconfiguration: bool, f: impl FnOnce(&mut Self) -> T) -> T;
}

impl PmapExt for crate::pac::PMAP {
  unsafe fn configure<T>(&mut self, allow_reconfiguration: bool, f: impl FnOnce(&mut Self) -> T) -> T {
    self.pmapkeyid.write(|w| unsafe { w.bits(UNLOCK_KEY) });

    self.pmapctl.modify(|_, w| w.pmaprecfg().bit(allow_reconfiguration));

    let v = f(self);

    self.pmapkeyid.write(|w| unsafe { w.bits(0) });

    v
  }
}
