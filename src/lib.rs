#![no_std]
#![feature(llvm_asm)]

extern crate embedded_hal as hal;
extern crate msp432p401r as pac;

pub use nb;

pub mod watchdog;
pub mod gpio;
pub mod time;
pub mod clock;
pub mod pcm;
pub mod flash;
pub mod timer;
pub mod eusci;
pub mod spi;
