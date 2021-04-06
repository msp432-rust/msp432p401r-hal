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

## Running the examples

Make sure you have OpenOCD and [GNU ARM Embedded toolchain](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads) installed and up-to-date

```shell
$ brew install openocd --HEAD
```

Setup Rust nightly and add the ARM build platform target:

```shell
$ rustup default nightly
$ rustup target add thumbv7em-none-eabihf
```

Clone and compile the project:

```shell
$ git clone https://github.com/msp432-rust/msp432p401r-hal.git
$ cd msp432p401r-hal
$ example=hello_world
$ cargo build --example $example
```

Open a OpenOCD server and leave it open in a terminal:

```shell
$ openocd
```

On a separate terminal, open the GDB client:

```shell
$ arm-none-eabi-gdb -q target/thumbv7em-none-eabihf/debug/examples/$example
$ (gdb) target extended-remote :3333
$ (gdb) load
$ (gdb) monitor arm semihosting enable
$ (gdb) continue
```
