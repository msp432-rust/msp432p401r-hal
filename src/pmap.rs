//! HAL library for PMAP (Port Mapping Controller) - MSP432P401R
pub use pac::PMAP;
pub use pac::pmap;
use cortex_m::interrupt;
use crate::gpio::{Alternate, Primary};

const UNLOCK_KEY: u16 = 0x2D52;
const OFFSET0: u16 = 0x0000;
const OFFSET1: u16 = 0x0008;

static mut PORT_MAP: PmapControl = PmapControl {map: None};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Mapping {
    NONE     =  0,
    UCA0CLK  =  1,
    UCA0RXD  =  2,
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
    pub const UCA0SOMI: Self  =  Self::UCA0RXD;
    pub const UCA0SIMO: Self  = Self::UCA0TXD;
    pub const UCB0SIMO: Self  = Self::UCB0SDA;
    pub const UCB0SOMI: Self  = Self::UCB0SCL;
    pub const UCA1SOMI: Self  = Self::UCA1RXD;
    pub const UCA1SIMO: Self  = Self::UCA1TXD;
    pub const UCA2SOMI: Self  = Self::UCA2RXD;
    pub const UCA2SIMO: Self  = Self::UCA2TXD;
    pub const UCB2SIMO: Self  = Self::UCB2SDA;
    pub const UCB2SOMI: Self  = Self::UCB2SCL;
    pub const C0OUT: Self     = Self::TA0CLK;
    pub const C1OUT: Self     = Self::TA1CLK;
    pub const SMCLK: Self     = Self::DMAE0;
}

pub struct PmapControl {
    map: Option<PMAP>,
}

pub trait PmapExt {
    fn constrain(self);
}

impl PmapExt for PMAP {
    fn constrain(self) {
        unsafe {
            PORT_MAP = PmapControl::new(self);
        };
    }
}

impl PmapControl {
    #[inline]
    fn new(pmp: PMAP) -> Self {
        PmapControl {
            map: Some(pmp),
        }
    }

    #[inline]
    fn configure(&mut self, allow_reconfiguration: bool) -> &Self {
        self.map.as_ref().unwrap().pmapctl.modify(|_, w| w.pmaprecfg().bit(allow_reconfiguration));
        self
    }

    #[inline]
    fn key_lock(&mut self, lock: bool) -> &Self{
        if lock {
            self.map.as_ref().unwrap().pmapkeyid.write(|w| unsafe {
                w.pmapkey().bits(0)
            });
        } else {
            self.map.as_ref().unwrap().pmapkeyid.write(|w| unsafe {
                w.pmapkey().bits(UNLOCK_KEY)
            });
        }
        self
    }
}

pub trait PortMap {
    fn remap(self, mapping: Mapping, allow_reconfiguration: bool) -> Self;
}

macro_rules! portmap {
    ($($PI_i:ident, $portx:ident, $pimapxy:ident, $offset:ident),*) => {
        $(
            use crate::gpio::$portx::$PI_i;

            impl PortMap for $PI_i<Alternate<Primary>> {

                #[inline]
                fn remap(self, mapping: Mapping, allow_reconfiguration: bool) -> Self {

                    unsafe{
                        PORT_MAP.key_lock(false);

                        PORT_MAP.configure(allow_reconfiguration);

                        interrupt::free(|_| {
                            PORT_MAP.map.as_ref().unwrap().$pimapxy.modify(|r, w|
                                w.pmapx().bits((r.pmapx().bits() & !(0x00FF << $offset)) | ((mapping as u16) << $offset))
                            );
                        });

                        PORT_MAP.key_lock(true);
                    };

                    self
                }
            }
        )*
    };
}

portmap! {
    P2_0, porta, p2map01, OFFSET0,
    P2_1, porta, p2map01, OFFSET1,
    P2_2, porta, p2map23, OFFSET0,
    P2_3, porta, p2map23, OFFSET1,
    P2_4, porta, p2map45, OFFSET0,
    P2_5, porta, p2map45, OFFSET1,
    P2_6, porta, p2map67, OFFSET0,
    P2_7, porta, p2map67, OFFSET1,
    P3_0, portb, p3map01, OFFSET0,
    P3_1, portb, p3map01, OFFSET1,
    P3_2, portb, p3map23, OFFSET0,
    P3_3, portb, p3map23, OFFSET1,
    P3_4, portb, p3map45, OFFSET0,
    P3_5, portb, p3map45, OFFSET1,
    P3_6, portb, p3map67, OFFSET0,
    P3_7, portb, p3map67, OFFSET1,
    P7_0, portd, p7map01, OFFSET0,
    P7_1, portd, p7map01, OFFSET1,
    P7_2, portd, p7map23, OFFSET0,
    P7_3, portd, p7map23, OFFSET1,
    P7_4, portd, p7map45, OFFSET0,
    P7_5, portd, p7map45, OFFSET1,
    P7_6, portd, p7map67, OFFSET0,
    P7_7, portd, p7map67, OFFSET1
}
