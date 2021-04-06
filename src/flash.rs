//! HAL library for Flash Control (FLCTL) - MSP432P401R
use pac::FLCTL;

pub struct FlcNotDefined;
pub struct FlcDefined;

pub trait State {}

impl State for FlcNotDefined {}
impl State for FlcDefined {}

pub trait FlashExt {
    fn constrain(self) -> FlashConfig<FlcNotDefined>;
}

impl FlashExt for FLCTL {
    fn constrain(self) -> FlashConfig<FlcNotDefined> {
        FlashConfig::new(self)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum FlashWaitStates {
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

impl FlashWaitStates {
    const fn val(&self) -> u8 {
        match *self {
            FlashWaitStates::_0Ws => 0x00,
            FlashWaitStates::_1Ws => 0x01,
            FlashWaitStates::_2Ws => 0x02,
            FlashWaitStates::_3Ws => 0x03,
            FlashWaitStates::_4Ws => 0x04,
            FlashWaitStates::_5Ws => 0x05,
            FlashWaitStates::_6Ws => 0x06,
            FlashWaitStates::_7Ws => 0x07,
            FlashWaitStates::_8Ws => 0x08,
            FlashWaitStates::_9Ws => 0x09,
            FlashWaitStates::_10Ws => 0x0A,
            FlashWaitStates::_11Ws => 0x0B,
            FlashWaitStates::_12Ws => 0x0C,
            FlashWaitStates::_13Ws => 0x0D,
            FlashWaitStates::_14Ws => 0x0E,
            FlashWaitStates::_15Ws => 0x0F,
        }
    }
}

pub struct FlashConfig<S: State> {
    flash: FLCTL,
    wait_states: FlashWaitStates,
    _state: S,
}

macro_rules! make_flashconf {
    ($conf:expr, $_state:expr) => {
        FlashConfig {
            flash: $conf.flash,
            wait_states: $conf.wait_states,
            _state: $_state,
        }
    };
}

impl FlashConfig<FlcNotDefined> {

    fn new(flash: FLCTL) -> Self {
        FlashConfig {
            flash,
            wait_states: FlashWaitStates::_0Ws,
            _state: FlcNotDefined,
        }
    }
}

impl FlashConfig<FlcDefined> {

    pub fn freeze(self) -> Self {
        self.set_rdctl_mask((self.wait_states.val() as u32) << 12, 0xFFFF0FFF);
        self
    }

    fn set_rdctl_mask(&self, value: u32, mask: u32) {

        self.flash.flctl_bank0_rdctl.modify(|r, w| unsafe {
            w.bits((r.bits() & mask as u32) | value as u32)
        });

        self.flash.flctl_bank1_rdctl.modify(|r, w| unsafe {
            w.bits((r.bits() & mask as u32) | value as u32)
        });
    }
}

impl<S> FlashConfig<S> where S: State {
    #[inline]
    pub fn set_waitstates(mut self, wait: FlashWaitStates)  -> FlashConfig<FlcDefined> {
        self.wait_states = wait;
        make_flashconf!(self, FlcDefined)
    }
}
