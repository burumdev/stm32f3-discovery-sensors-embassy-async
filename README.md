# STM32 F3 Discovery LSM303AGR Async

Running LSM303AGR magnetometer, temperature and accelerometer asynchronously with Embassy framework.

## Description

Embassy is an async runtime and framework for various embedded systems.
The aim of this demo is to share a single peripheral driver instance of LSM303AGR magnetometer-accelerometer device between async tasks.

Device can be found on the stm32 f3 discovery development board and it's already hardwired to the MCU. We utilize i2c bus for communication with the device.

We then read the sensor values of temperature in degrees celcius,
EMT or earth's magnetic field in nanopascals and 3 axis acceleration in milli Gs. Tasks should read these values concurrently, in varied intervals like once 3 seconds, once 2048 ms and once 777 ms.

## Hardware

Project is developed on STM32 F3 Discovery development board with Arm Cortex 4m core. Newest versions of this board has the LSM303AGR device.
Earlier versions might have an LSM303DLHC also from ST Microelectronics, but it's a completely different beast and won't work with the driver or i2c settings used in this demo.

## Libraries used
* [embassy-rs](https://github.com/embassy-rs/embassy) provides the async runtime tuned for embedded hardware ecosystem and behaviour. It's also a framework, so in this
demo it also provides the PAC (low level peripheral access), HAL (high level hardware abstractions that humans can relate to) and timer support.

* [lsm303agr-rs](https://github.com/eldruin/lsm303agr-rs) driver for the magnetometer and accelerometer which supports asynchronous operation.

* [probe-rs](https://github.com/probe-rs/probe-rs) awesome command-line tool for flashing firmware and debugging embedded devices.

* [defmt](https://github.com/knurling-rs/defmt) console.log()™ equivalent for embedded devices.

* [static-cell](https://crates.io/crates/static_cell) allocates space for global variables in compile time and allows initializing them in runtime,
doing lazy one time initialization and returning a static mutable reference. Thus globalizing local variables with ease.

## To build and run you need

* Rust build system rustup and cargo.
* probe-rs firmware flashing and debugging tool (can be installed via cargo binary)
* Electrical connections on the stm32f3-discovery board is not necessary. LSM303AGR device is hardwired to the MCU. Just connect it via USB.

### Executing program

Connect stm32f3-discovery via USB and:

```
cargo run
```

## Authors

Barış Ürüm
