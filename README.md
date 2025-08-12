# STM32 F3 Discovery Async Sensors
Running LSM303AGR MEMS magnetometer-accelerometer, I3G4250D MEMS gyroscope (both from STMicroelectronics) and
DS3231 Real Time Clock (Analog Devices) asynchronously with Embassy framework. Development board is STM32F3DISCOVERY.


https://github.com/user-attachments/assets/d5785416-2ce2-4344-9fa2-ffbb7d27dd35


## Description
Embassy is an async runtime and framework for various embedded systems.
The aim of this demo is to run magneto and gyro devices found on the discovery development board and an external RTC clock device
with single peripheral SPI and shared I2C bus configurations concurrently.

Magneto and gyro devices can be found on the STM32F3DISCOVERY development board already hardwired to the MCU.
DS3231 RTC is connected externally to the same I2C bus of LSM303AGR magnetometer.
We utilize SPI bus for the gyroscope.

We then read the sensor values of temperature in degrees celcius,
EMT or earth's magnetic field in nanopascals, 3 axis acceleration in milli Gs and datetime values from the RTC clock asynchronously.
Gyroscope values are read synchronously (they block everything) but SPI bus is ran at a high clock rate and timing of the reads is also async.
Tasks should read the sensor values concurrently, in varied intervals like once 3 seconds, once 2048 ms and once 777 ms.
RTC values are read with an async interrupt event handler.

My original aim was to run both LSM303AGR and I3G4250D on the same shared I2C bus by
using synchronization primitives for both busses and drivers across async tasks.
See [shared_bus](https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/shared_bus.rs) example of Embassy framework.
But the I3G4250D driver though works perfectly fine, doesn't support I2C bus at this time.
So I added the DS3231 RTC clock to the project to share the I2C bus with LSM303AGR.

## Hardware
Project is developed on STM32 F3 Discovery development board with Arm Cortex m4 core.
Newest versions of this board have the LSM303AGR and I3G44250D devices.
Earlier versions might have an LSM303DLHC as magneto and L3GD20 as gyro also from STMicroelectronics,
but they're completely different beasts and probably won't work with the drivers or comm settings used in this demo.
DS3231 RTC from Analog Devices is added externally to the same I2C bus pins of the LSM303AGR for SCL and SDA.

A sketch of the bus connections can be seen below. Note that some of the connections are already present on the board. VCC and other connections omitted.
<img width="1164" height="848" alt="STM32F3Discovery-sensors-async" src="https://github.com/user-attachments/assets/058ae92b-e0ea-4eb7-a24d-02973e031eba" />

## Libraries used
* [embassy-rs](https://github.com/embassy-rs/embassy) provides the async runtime tuned for embedded hardware ecosystem and behaviour. It's also a framework and in this
demo it also provides the PAC (low level peripheral access), HAL (high level hardware abstractions that humans can relate to) and timer support.

* [lsm303agr-rs](https://github.com/eldruin/lsm303agr-rs) driver for the magnetometer and accelerometer. Supports async operation.

* [ds3231-rs](https://github.com/liebman/ds3231-rs) driver for the realtime clock. Shares the same I2C bus with magneto and supports async operation.

* [i3g4250d](https://docs.rs/i3g4250d/latest/i3g4250d/) driver for the gyroscope. Works synchronously. But SPI works in 1MHz so not a problem at all.

* [probe-rs](https://github.com/probe-rs/probe-rs) awesome command-line tool for flashing firmware and debugging embedded devices.

* [defmt](https://github.com/knurling-rs/defmt) console.log()™ equivalent for embedded devices.

* [static-cell](https://crates.io/crates/static_cell) allocates space for global variables in compile time and allows initializing them in runtime,
doing lazy one time initialization and returning a static mutable reference. Thus globalizing local variables with ease.

## To build and run you need

* Rust build system rustup and cargo.
    * https://www.rust-lang.org/tools/install
* Arm Cortex M4F cross compilation target libraries. Add them with:
    * `rustup target add thumbv7em-none-eabihf`
* probe-rs firmware flashing and debugging tool (can be installed via cargo binary)
    * `cargo install prope-rs`
* Electrical connections from the stm32f3-discovery board are:
    * GND -> DS3231 GND
    * 3V3 -> DS3231 VCC
    * PA0 -> DS3231 SQW (Used for interrupts)
    * PB6 -> DS3231 SCL (I2C serial clock)
    * PB7 -> DS3231 SDA (I2C serial data)

### Executing program

Connect stm32f3-discovery via USB, wire-up the DS3231 and:

```
cargo run
```

## License
MIT

## Authors
Barış Ürüm
