# STM32 f3 Discovery LSM303AGR Async

Running LSM303AGR magnetometer, temperature and accelerometer asynchronously with Embassy framework.

## Description

Embassy is an async runtime and framework for various embedded systems.
The aim of this demo is to share a single peripheral driver instance of LSM303AGR magnetometer-accelerometer device that's found on stm32f3-discovery between async tasks.
Device is utilized via i2c connection. Then read the sensor values of temperature in degrees celcius, EMT - earth's magnetic field in nanopascals and acceleration in milli Gs
in varied intervals like once 3 seconds, once 2048 ms and once 777 ms concurrently.

## Hardware
Project is developed on an stm32 f3 discovery development board with Arm cortex 4m core.

## Dependencies

* Rust build system rustup and cargo.
* probe-rs firmware flashing and debugging tool (can be installed via cargo binary)
* Electrical connections on the stm32f3-discovery board is not necessary LSM303AGR device is hardwired to the MCU.

### Executing program

Connect stm32f3-discovery via USB and:

```
cargo run
```

## Authors

Barış Ürüm
