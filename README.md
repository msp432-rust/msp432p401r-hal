# msp432p401r-hal ![crates.io](https://img.shields.io/crates/v/msp432p401r-hal.svg)

Hardware Abstraction Layer for MSP432P401R microcontroller

Currently under development:
- [X] Watchdog
- [X] GPIO
- [X] Clock
- [X] PCM
- [X] Flash Control
- [X] TimerA
- [X] Timer32
- [X] PWM (TimerA)
- [ ] Port MAP
- [ ] ADC
- [ ] DMA
- [ ] RTC
- [ ] SPI - EUSCI
- [ ] I²C - EUSCI
- [ ] UART - EUSCI
- [ ] Cap. Touch IO
- [ ] CRC32
- [ ] AES256
- [ ] Shared Reference (REF A)
- [ ] Comparator
- [ ] LCD Control
- [ ] FPU
- [ ] Random Seed
- [ ] Reset Controller
- [ ] System Controller
- [ ] Power Supply System
- [ ] Cortex M4 Periph 

# How to run the examples?

Setup Rust beta and add the ARM build platform target:

$ rustup default beta

$ rustup target add thumbv7em-none-eabihf

Clone and compile the project:

$ git clone https://github.com/msp432-rust/msp432p401r-hal.git

$ cd msp432p401r-hal

$ cargo build

$ cargo build --example "example name"

Open a OpenOCD server and leave it open in a terminal:

$ openocd

On a separate terminal, open the GDB client:

$ arm-none-eabi-gdb -q target/thumbv7em-none-eabihf/debug/examples/"example name"

$ (gdb) target remote :3333

$ (gdb) load

$ (gdb) monitor arm semihosting enable

$ (gdb) continue
