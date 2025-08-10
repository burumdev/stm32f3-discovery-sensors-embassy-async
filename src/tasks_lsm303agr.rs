use defmt::*;
use embassy_time::Timer;
use lsm303agr::Error as Lsm303agrError;
use lsm303agr::{Lsm303agr, interface::I2cInterface, mode::MagOneShot};

use embassy_stm32::{i2c::I2c, mode::Async};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;

type Magnetometer =
    Lsm303agr<I2cInterface<I2cDevice<'static, NoopRawMutex, I2c<'static, Async>>>, MagOneShot>;
use crate::SharedI2CBusMutex;

fn get_lsm303agr_error_text<E>(err: &Lsm303agrError<E>) -> &'static str {
    match err {
        Lsm303agrError::Comm(_) => "I2C communication error.",
        Lsm303agrError::InvalidInputData => "Invalid input data.",
    }
}

async fn read_temperature(mag_driver: &mut Magnetometer) {
    match mag_driver.temperature().await {
        Ok(temperature) => {
            info!("Magnetometer temperature read successful.");
            info!(
                "Current mag temperature is: {} °C",
                temperature.degrees_celsius()
            );
        }
        Err(err) => {
            error!(
                "ERROR reading mag temperature: {}",
                get_lsm303agr_error_text(&err)
            );
        }
    }
}
#[embassy_executor::task]
pub async fn read_mag_temperature_every_n_seconds(
    i2c_bus: &'static SharedI2CBusMutex,
    n_seconds: u64,
) {
    let shared_i2c_device = I2cDevice::new(i2c_bus);
    let mut lsm303agr = Lsm303agr::new_with_i2c(shared_i2c_device);
    info!("*** MAG TEMPERATURE THE FIRST TIME ***");

    read_temperature(&mut lsm303agr).await;

    loop {
        Timer::after_secs(n_seconds).await;
        info!("\n*** MAG TEMPERATURE EVERY {} SECONDS ***", n_seconds);
        read_temperature(&mut lsm303agr).await;
    }
}

async fn read_magnetometer(lsm303agr: &mut Magnetometer) {
    match lsm303agr.magnetic_field().await {
        Ok(mag) => {
            info!("Magnetometer read successful.");
            info!(
                "Earth's magnetic field: x = {} nanoteslas, y = {} nanoteslas, z = {} nanoteslas",
                mag.x_nt(),
                mag.y_nt(),
                mag.z_nt()
            );
        }
        Err(err) => {
            error!(
                "ERROR reading magnetometer: {}",
                get_lsm303agr_error_text(&err)
            );
        }
    }
}
#[embassy_executor::task]
pub async fn read_magnetometer_every_n_milliseconds(
    i2c_bus: &'static SharedI2CBusMutex,
    n_millis: u64,
) {
    let shared_i2c_device = I2cDevice::new(i2c_bus);
    let mut lsm303agr = Lsm303agr::new_with_i2c(shared_i2c_device);
    info!("*** EMF THE FIRST TIME ***");
    read_magnetometer(&mut lsm303agr).await;

    loop {
        Timer::after_millis(n_millis).await;
        info!("\n*** EMF EVERY {} MILLISECONDS ***", n_millis);
        read_magnetometer(&mut lsm303agr).await;
    }
}

async fn read_accelerometer(lsm303agr: &mut Magnetometer) {
    match lsm303agr.acceleration().await {
        Ok(accel) => {
            info!("Acceleration read successful.");
            info!(
                "3D acceleration x = {} milli Gs, y = {} milli Gs, z = {} milli Gs",
                accel.x_mg(),
                accel.y_mg(),
                accel.z_mg(),
            );
        }
        Err(err) => {
            error!(
                "ERROR reading accelerometer: {}",
                get_lsm303agr_error_text(&err)
            );
        }
    }
}
#[embassy_executor::task]
pub async fn read_accelerometer_every_n_milliseconds(
    i2c_bus: &'static SharedI2CBusMutex,
    n_millis: u64,
) {
    let shared_i2c_device = I2cDevice::new(i2c_bus);
    let mut lsm303agr = Lsm303agr::new_with_i2c(shared_i2c_device);
    info!("*** ACCELERATION THE FIRST TIME ***");
    read_accelerometer(&mut lsm303agr).await;

    loop {
        Timer::after_millis(n_millis).await;
        info!("\n*** ACCELERATION EVERY {} MILLISECONDS ***", n_millis);
        read_accelerometer(&mut lsm303agr).await;
    }
}
