//! HAL library for Flash Control (FLCTL) - MSP432P401R
use pac::FLCTL;
use crate::common::{Constrain, NotDefined, Defined};

impl Constrain<FlashControl<NotDefined>> for FLCTL {
    fn constrain(self) -> FlashControl<NotDefined> {
        FlashControl::<NotDefined>::new(self)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum FlashWaitStates {
    _0 = 0x00,
    _1 = 0x01,
    _2 = 0x02,
    _3 = 0x03,
    _4 = 0x04,
    _5 = 0x05,
    _6 = 0x06,
    _7 = 0x07,
    _8 = 0x08,
    _9 = 0x09,
    _10 = 0x0A,
    _11 = 0x0B,
    _12 = 0x0C,
    _13 = 0x0D,
    _14 = 0x0E,
    _15 = 0x0F,
}

pub struct FlashControl<State> {
    flash: FLCTL,
    wait_states: FlashWaitStates,
    _state: State,
}

impl FlashControl<NotDefined> {
    fn new(flash: FLCTL) -> Self {
        FlashControl::<NotDefined>{
            flash,
            wait_states: FlashWaitStates::_0,
            _state: NotDefined,
        }
    }
}

impl FlashControl<Defined> {
    pub fn freeze(self) -> Self {
        self.set_read_control((self.wait_states as u32) << 12);
        self
    }

    fn set_read_control(&self, value: u32) {
        const MASK: u32 = 0xFFFF_0FFF;
        self.flash.flctl_bank0_rdctl.modify(|r, w| unsafe {
            w.bits((r.bits() & MASK) | value)
        });

        self.flash.flctl_bank1_rdctl.modify(|r, w| unsafe {
            w.bits((r.bits() & MASK) | value)
        });
    }
}

impl<State> FlashControl<State>{
    #[inline]
    /// Set Wait States for the flash memory read operations (depends on the current clock speed)
    pub fn set_waitstates(self, wait: FlashWaitStates) -> FlashControl<Defined> {
        FlashControl::<Defined> {
            flash: self.flash,
            wait_states: wait,
            _state: Defined,
        }
    }
}
