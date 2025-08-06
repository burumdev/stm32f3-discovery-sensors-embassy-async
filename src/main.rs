#![no_main]
#![no_std]

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    i2c::{self, I2c},
    mode::Async,
    peripherals,
    time::Hertz,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex};
use embassy_time::Timer;

use lsm303agr::{Error as Lsm303agrError, Lsm303agr, interface::I2cInterface, mode::MagOneShot};

use static_cell::StaticCell;

use defmt::*;
use defmt_rtt as _;

use panic_probe as _;

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

type Magnetometer = Lsm303agr<I2cInterface<I2c<'static, Async>>, MagOneShot>;
type MagnetoMutex = mutex::Mutex<CriticalSectionRawMutex, Magnetometer>;

fn get_lsm303agr_error_text<E>(err: &Lsm303agrError<E>) -> &'static str {
    match err {
        Lsm303agrError::Comm(_) => "I2C communication error.",
        Lsm303agrError::InvalidInputData => "Invalid input data.",
    }
}

async fn read_temperature(lsm303agr: &'static MagnetoMutex) {
    match lsm303agr.lock().await.temperature().await {
        Ok(temperature) => {
            info!("Temperature read successful.");
            info!(
                "Current temperature is: {} °C",
                temperature.degrees_celsius()
            );
        }
        Err(err) => {
            let err_txt = match err {
                Lsm303agrError::Comm(_) => "I2C communication error.",
                Lsm303agrError::InvalidInputData => "Invalid input data.",
            };
            error!("ERROR reading temperature: {}", err_txt);
        }
    }
}
#[embassy_executor::task]
async fn read_temperature_every_n_seconds(lsm303agr: &'static MagnetoMutex, n_seconds: u64) {
    info!("\n*** TEMPERATURE THE FIRST TIME ***");
    read_temperature(lsm303agr).await;

    loop {
        Timer::after_secs(n_seconds).await;
        info!("\n*** TEMPERATURE EVERY {} SECONDS ***", n_seconds);
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
async fn read_magnetometer_every_n_milliseconds(lsm303agr: &'static MagnetoMutex, n_millis: u64) {
    info!("\n*** EMF THE FIRST TIME ***");
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
async fn read_accelerometer_every_n_milliseconds(lsm303agr: &'static MagnetoMutex, n_millis: u64) {
    info!("\n*** ACCELERATION THE FIRST TIME ***");
    read_accelerometer(lsm303agr).await;

    loop {
        Timer::after_millis(n_millis).await;
        info!("\n*** ACCELERATION EVERY {} MILLISECONDS ***", n_millis);
        read_accelerometer(lsm303agr).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Does defmt work? Yes it does!");

    let peris = embassy_stm32::init(Default::default());

    let i2c = I2c::new(
        peris.I2C1,
        // These pins are hardwired to the onboard magnetometer LSM303AGR on the stm32f3 discovery
        peris.PB6,
        peris.PB7,
        Irqs,
        // DMA channels for I2c TX DMA (Sender) and RX DMA (Receiver)
        peris.DMA1_CH6,
        peris.DMA1_CH7,
        // We use normal transmission speed
        Hertz(100_000),
        Default::default(),
    );

    static LSM303AGR_CELL: StaticCell<MagnetoMutex> = StaticCell::new();
    let lsm303agr = Lsm303agr::new_with_i2c(i2c);
    let lsm303agr = LSM303AGR_CELL.init(mutex::Mutex::new(lsm303agr));

    spawner
        .spawn(read_temperature_every_n_seconds(lsm303agr, 3))
        .unwrap();

    spawner
        .spawn(read_magnetometer_every_n_milliseconds(lsm303agr, 2048))
        .unwrap();

    spawner
        .spawn(read_accelerometer_every_n_milliseconds(lsm303agr, 777))
        .unwrap();
}
