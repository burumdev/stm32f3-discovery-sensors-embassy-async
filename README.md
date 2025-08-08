# STM32 F3 Discovery Async Sensors
Running LSM303AGR MEMS magnetometer-accelerometer and I3G4250D MEMS gyroscope (both from STMicroelectronics) asynchronously with Embassy async embedded framework.

## Description
Embassy is an async runtime and framework for various embedded systems.
The aim of this demo is to share single peripheral driver instances of LSM303AGR and I3G44250D
devices between async tasks and read their values with minimal MCU utilization.

Devices can be found on the stm32 f3 discovery development board already hardwired to the MCU.
We utilize I2C bus for communication with the magnetometer and SPI bus for gyroscope.

We then read the sensor values of temperature in degrees celcius,
EMT or earth's magnetic field in nanopascals and 3 axis acceleration in milli Gs asynchronously.
Gyroscope values are read synchronously (they block everything) but SPI bus is ran at a high clock rate and timing of the reads is async.
Tasks should read these values concurrently, in varied intervals like once 3 seconds, once 2048 ms and once 777 ms.

My original aim was to run both LSM303AGR and I3G4250D on the same shared I2C bus by
using synchronization primitives for both busses and drivers across async tasks.
See [shared_bus](https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/shared_bus.rs) example of Embassy framework.
But the I3G4250D driver though works perfectly fine, doesn't support I2C bus at this time.

In the future I might add an I2C device like DS3231 Real Time Clock to this demo to achieve shared bus functionality.

## Hardware
Project is developed on STM32 F3 Discovery development board with Arm Cortex m4 core. Newest versions of this board have the LSM303AGR and I3G44250D devices.
Earlier versions might have an LSM303DLHC as magneto and L3GD20 as gyro also from STMicroelectronics,
but they're completely different beasts and probably won't work with the drivers or comm settings used in this demo.

## Libraries used
* [embassy-rs](https://github.com/embassy-rs/embassy) provides the async runtime tuned for embedded hardware ecosystem and behaviour. It's also a framework and in this
demo it also provides the PAC (low level peripheral access), HAL (high level hardware abstractions that humans can relate to) and timer support.

* [lsm303agr-rs](https://github.com/eldruin/lsm303agr-rs) driver for the magnetometer and accelerometer which supports asynchronous operation.

* [i3g4250d](https://docs.rs/i3g4250d/latest/i3g4250d/) driver for the gyroscope. Works synchronously. But SPI works in 1MHz so not a problem at all.

* [probe-rs](https://github.com/probe-rs/probe-rs) awesome command-line tool for flashing firmware and debugging embedded devices.

* [defmt](https://github.com/knurling-rs/defmt) console.log()™ equivalent for embedded devices.

* [static-cell](https://crates.io/crates/static_cell) allocates space for global variables in compile time and allows initializing them in runtime,
doing lazy one time initialization and returning a static mutable reference. Thus globalizing local variables with ease.

## To build and run you need

* Rust build system rustup and cargo.
* probe-rs firmware flashing and debugging tool (can be installed via cargo binary)
* Electrical connections on the stm32f3-discovery board is not necessary. The devices are hardwired to the MCU. Just connect the stm32f3 via USB.

### Executing program

Connect stm32f3-discovery via USB and:

```
cargo run
```

## License
MIT

## Authors
Barış Ürüm
