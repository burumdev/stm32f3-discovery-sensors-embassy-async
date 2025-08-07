use defmt::*;
use embassy_time::Timer;
use lsm303agr::Error as Lsm303agrError;

use crate::MagnetoMutex;

fn get_lsm303agr_error_text<E>(err: &Lsm303agrError<E>) -> &'static str {
    match err {
        Lsm303agrError::Comm(_) => "I2C communication error.",
        Lsm303agrError::InvalidInputData => "Invalid input data.",
    }
}

async fn read_temperature(lsm303agr: &'static MagnetoMutex) {
    match lsm303agr.lock().await.temperature().await {
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
    lsm303agr: &'static MagnetoMutex,
    n_seconds: u64,
) {
    info!("*** MAG TEMPERATURE THE FIRST TIME ***");
    read_temperature(lsm303agr).await;

    loop {
        Timer::after_secs(n_seconds).await;
        info!("\n*** MAG TEMPERATURE EVERY {} SECONDS ***", n_seconds);
        read_temperature(lsm303agr).await;
    }
}

async fn read_magnetometer(lsm303agr: &'static MagnetoMutex) {
    match lsm303agr.lock().await.magnetic_field().await {
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
    lsm303agr: &'static MagnetoMutex,
    n_millis: u64,
) {
    info!("*** EMF THE FIRST TIME ***");
    read_magnetometer(lsm303agr).await;

    loop {
        Timer::after_millis(n_millis).await;
        info!("\n*** EMF EVERY {} MILLISECONDS ***", n_millis);
        read_magnetometer(lsm303agr).await;
    }
}

async fn read_accelerometer(lsm303agr: &'static MagnetoMutex) {
    match lsm303agr.lock().await.acceleration().await {
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
    lsm303agr: &'static MagnetoMutex,
    n_millis: u64,
) {
    info!("*** ACCELERATION THE FIRST TIME ***");
    read_accelerometer(lsm303agr).await;

    loop {
        Timer::after_millis(n_millis).await;
        info!("\n*** ACCELERATION EVERY {} MILLISECONDS ***", n_millis);
        read_accelerometer(lsm303agr).await;
    }
}
