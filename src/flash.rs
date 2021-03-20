//! HAL library for Flash Control (FLCTL) - MSP432P401R
use pac::FLCTL;

/// Typestate for `FlctlConfig` that represents unconfigured FLCTL
pub struct FlcNotDefined;

/// Typestate for `FlctlConfig` that represents a configured FLCTL
pub struct FlcDefined;

pub trait State {}

impl State for FlcNotDefined {}

impl State for FlcDefined {}

#[derive(Copy, Clone, PartialEq)]
pub enum FlWaitSts {
    _0Ws,
    _1Ws,
    _2Ws,
    _3Ws,
    _4Ws,
    _5Ws,
    _6Ws,
    _7Ws,
    _8Ws,
    _9Ws,
    _10Ws,
    _11Ws,
    _12Ws,
    _13Ws,
    _14Ws,
    _15Ws,
}

impl FlWaitSts {
    /// Numerical frequency
    #[inline(always)]
    pub fn val(&self) -> u8 {
        match *self {
            FlWaitSts::_0Ws => 0x00,
            FlWaitSts::_1Ws => 0x01,
            FlWaitSts::_2Ws => 0x02,
            FlWaitSts::_3Ws => 0x03,
            FlWaitSts::_4Ws => 0x04,
            FlWaitSts::_5Ws => 0x05,
            FlWaitSts::_6Ws => 0x06,
            FlWaitSts::_7Ws => 0x07,
            FlWaitSts::_8Ws => 0x08,
            FlWaitSts::_9Ws => 0x09,
            FlWaitSts::_10Ws => 0x0A,
            FlWaitSts::_11Ws => 0x0B,
            FlWaitSts::_12Ws => 0x0C,
            FlWaitSts::_13Ws => 0x0D,
            FlWaitSts::_14Ws => 0x0E,
            FlWaitSts::_15Ws => 0x0F,
        }
    }
}

/// Builder object that configures the Flash Control (FLCTL)
pub struct FlashConfig<'a, S: State> {
    periph: &'a pac::flctl::RegisterBlock,
    _state: S,
}

impl<'a, S> FlashConfig<'a, S> where S: State {
    /// Converts FLCTL into a fresh, unconfigured FLCTL builder object
    pub fn new() -> FlashConfig<'a, FlcDefined> {
        let flctl = unsafe { &*FLCTL::ptr() };

        FlashConfig {
            periph: flctl,
            _state: FlcDefined,
        }
    }

    fn set_rdctl_mask(&self, value: u32, mask: u32) {
        self.periph.flctl_bank0_rdctl.modify(|r, w| unsafe {
            w.bits((r.bits() & mask as u32) | value as u32)
        });

        self.periph.flctl_bank1_rdctl.modify(|r, w| unsafe {
            w.bits((r.bits() & mask as u32) | value as u32)
        });
    }
}

impl<'a> FlashConfig<'a, FlcDefined> {
    pub fn set_flwaitst(&self, wait: FlWaitSts) {
        self.set_rdctl_mask((wait.val() as u32) << 12, 0xFFFF0FFF);
    }
}
