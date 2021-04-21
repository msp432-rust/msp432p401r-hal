#![no_std]
#![feature(llvm_asm)]

extern crate embedded_hal as hal;
extern crate msp432p401r as pac;

pub mod common;
pub mod clock;
pub mod flash;
pub mod gpio;
pub mod pcm;
pub mod pmap;
pub mod serial;
pub mod time;
pub mod timer;
pub mod watchdog;
