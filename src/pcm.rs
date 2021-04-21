//! HAL library for PCM (Power Control Manager) - MSP432P401R
use pac::PCM;
use pac::pcm::pcmctl0::{AMR_A, CPM_A};
use cortex_m::{asm::delay, interrupt};
use crate::common::{Constrain, NotDefined, Defined};
use CoreVoltageSelection::*;
use PowerMode::*;

const CSKEY: u16 = 0x695A;

impl Constrain<PcmControl<NotDefined>> for PCM {
    fn constrain(self) -> PcmControl<NotDefined> {
        PcmControl::<NotDefined>::new(self)
    }
}

/// Core Voltage Selection
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CoreVoltageSelection {
    LdoLowPower          = 0x00,
    Ldo                  = 0x01,
    DcDcLowPower         = 0x04,
    DcDc                 = 0x05,
    LowFrequencyLowPower = 0x08,
    LowFrequency         = 0x09,
}

/// Power Mode Selection
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PowerMode {
    Active      = 0x00,
    LowPower0   = 0x01,
    LowPower3   = 0x02,
    LowPower3_5 = 0x03,
    LowPower4   = 0x04,
    LowPower4_5 = 0x05,
}

pub struct PcmControl<State>{
    pcm: PCM,
    source: CoreVoltageSelection,
    _state: State,
}

impl PcmControl<NotDefined> {
    fn new(pcm: PCM) -> Self {
        PcmControl::<NotDefined>{
            pcm,
            source: LdoLowPower,
            _state: NotDefined,
        }
    }
}

impl <State>PcmControl<State> {
    #[inline]
    /// Set the core voltage
    pub fn set_core_voltage(self, selection: CoreVoltageSelection) -> PcmControl<Defined> {
        PcmControl::<Defined> {
            pcm: self.pcm,
            source: selection,
            _state: Defined,
        }
    }
}

impl PcmControl<Defined> {
    /// Hold the core voltage selection
    #[inline]
    pub fn freeze(mut self) -> Self {
        let mut source_state: CoreVoltageSelection = self.get_core_voltage();

        while source_state != self.source {
            if source_state == DcDc || source_state == LowFrequency {
                source_state = Ldo;
            } else if source_state == DcDcLowPower || source_state == LowFrequencyLowPower {
                source_state = LdoLowPower;
            } else if source_state == Ldo &&
                self.source != DcDc && self.source != LowFrequency {
                source_state = LdoLowPower;
            } else if source_state == LdoLowPower &&
                self.source != DcDcLowPower && self.source != LowFrequencyLowPower {
                source_state = Ldo;
            } else {
                source_state = self.source;
            }

            self.update_voltage(source_state);
            source_state = self.get_core_voltage();
        }
        self
    }

    #[inline]
    fn wait_busy(&self) {
        while self.pcm.pcmctl1.read().pmr_busy().bit() {
            delay(10);
        }
    }

    #[inline]
    fn update_voltage(&mut self, source: CoreVoltageSelection) {
        self.wait_busy();
        interrupt::free(|_| {
            self.pcm.pcmctl0.write(|w| unsafe {
                w.pcmkey().bits(CSKEY).amr().bits(source as u8)
            });

            self.pcm.pcmctl0.write(|w| unsafe{ w.pcmkey().bits(!CSKEY) });
        });
        self.wait_busy();
    }

    #[inline]
    pub fn get_core_voltage(&self) -> CoreVoltageSelection {
        match self.pcm.pcmctl0.read().amr().variant().unwrap() {
            AMR_A::AMR_0 => LdoLowPower,
            AMR_A::AMR_1 => Ldo,
            AMR_A::AMR_4 => DcDcLowPower,
            AMR_A::AMR_5 => DcDc,
            AMR_A::AMR_8 => LowFrequencyLowPower,
            AMR_A::AMR_9 => LowFrequency,
        }
    }

    #[inline]
    pub fn get_power_mode(&self) -> PowerMode {
        match self.pcm.pcmctl0.read().cpm().variant().unwrap() {
            CPM_A::CPM_0 | CPM_A::CPM_1 | CPM_A::CPM_4 | CPM_A::CPM_5 |
            CPM_A::CPM_8 | CPM_A::CPM_9 => Active,
            CPM_A::CPM_16 | CPM_A::CPM_17 | CPM_A::CPM_20 | CPM_A::CPM_21 |
            CPM_A::CPM_24 | CPM_A::CPM_25 => LowPower0,
            CPM_A::CPM_32 => LowPower3,
        }
    }
}


