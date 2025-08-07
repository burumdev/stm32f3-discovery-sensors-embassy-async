use defmt::*;
use embassy_time::Timer;

use crate::GyroMutex;

use embassy_stm32::spi::Error as SpiError;
use i3g4250d::I16x3;

const CAL_SAMPLE_SIZE: usize = 1000;

async fn calibrate_gyro(i3g4250d: &'static GyroMutex) -> Result<I16x3, &'static str> {
    info!(
        "Calibrating I3G4250d gyroscope with {} samples. ETA is 10 seconds. Hold the gyro flat to the earth's surface.",
        CAL_SAMPLE_SIZE
    );
    let mut average = I16x3 { x: 0, y: 0, z: 0 };

    let mut x_sum: i32 = 0;
    let mut y_sum: i32 = 0;
    let mut z_sum: i32 = 0;

    for _ in 0..CAL_SAMPLE_SIZE {
        match i3g4250d.lock().await.gyro() {
            Ok(gyro_data) => {
                // Skip samples that deviate too much from national average
                // These are most probably electrical noise or SPI noise
                if (gyro_data.x - average.x).abs() > 50
                    || (gyro_data.y - average.y).abs() > 50
                    || (gyro_data.z - average.z).abs() > 50
                {
                    continue; // Skip this sample
                }

                x_sum += gyro_data.x as i32;
                y_sum += gyro_data.y as i32;
                z_sum += gyro_data.z as i32;

                average.x = (x_sum / CAL_SAMPLE_SIZE as i32) as i16;
                average.y = (y_sum / CAL_SAMPLE_SIZE as i32) as i16;
                average.z = (z_sum / CAL_SAMPLE_SIZE as i32) as i16;
            }
            Err(_) => return Err("ERROR while calibrating gyro! Could not read gyro data."),
        }

        Timer::after_millis(10).await;
    }

    Ok(average)
}

fn get_spi_error_text(err: &SpiError) -> &'static str {
    match err {
        SpiError::Framing => "SPI invalid framing",
        SpiError::Crc => "SPI CRC check error. Is CRC even enabled?",
        SpiError::ModeFault => "SPI mode faulty",
        SpiError::Overrun => "SPI overrun",
    }
}

async fn read_gyro(i3g4250d: &'static GyroMutex, cal_offsets: &I16x3) {
    match i3g4250d.lock().await.gyro() {
        Ok(gyro_all) => {
            let calibrated = I16x3 {
                x: gyro_all.x - cal_offsets.x,
                y: gyro_all.y - cal_offsets.y,
                z: gyro_all.z - cal_offsets.z,
            };
            info!("Gyro read successful.");
            info!(
                "Direction is x = {}, y = {}, z = {}",
                calibrated.x, calibrated.y, calibrated.z
            );
        }
        Err(err) => {
            error!("ERROR reading gyro values: {}", get_spi_error_text(&err));
        }
    }
}
#[embassy_executor::task]
pub async fn read_gyro_every_n_milliseconds(i3g4250d: &'static GyroMutex, n_millis: u64) {
    match calibrate_gyro(i3g4250d).await {
        Ok(cal_offsets) => {
            info!("*** GYRO THE FIRST TIME ***");
            read_gyro(i3g4250d, &cal_offsets).await;

            loop {
                Timer::after_millis(n_millis).await;
                info!("\n*** GYRO EVERY {} MILLISECONDS ***", n_millis);
                read_gyro(i3g4250d, &cal_offsets).await;
            }
        }
        Err(e) => {
            error!("{}", e);
        }
    }
}

async fn read_gyro_temperature(i3g4250d: &'static GyroMutex) {
    match i3g4250d.lock().await.temp() {
        Ok(temperature) => {
            info!("Gyro temperature read successful.");
            info!("Current gyro temperature is {} °C", temperature);
        }
        Err(err) => {
            error!(
                "ERROR reading gyro temperature values: {}",
                get_spi_error_text(&err)
            );
        }
    }
}
#[embassy_executor::task]
pub async fn read_gyro_temperature_every_n_seconds(i3g4250d: &'static GyroMutex, n_seconds: u64) {
    info!("*** GYRO TEMPERATURE THE FIRST TIME ***");
    read_gyro_temperature(i3g4250d).await;

    loop {
        Timer::after_secs(n_seconds).await;
        info!("\n*** GYRO TEMPERATURE EVERY {} SECONDS ***", n_seconds);
        read_gyro_temperature(i3g4250d).await;
    }
}
