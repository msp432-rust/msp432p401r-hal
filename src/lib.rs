#![no_std]
#![feature(llvm_asm)]

extern crate msp432p401r as pac;
extern crate embedded_hal as hal;

pub mod watchdog;
pub mod gpio;
pub mod time;
pub mod clock;
pub mod pcm;
pub mod flctl;
