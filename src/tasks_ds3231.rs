use defmt::*;

use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_stm32::exti::ExtiInput;

use chrono::{Datelike, NaiveDate, Timelike};

use crate::SharedI2CBusMutex;
use ds3231::{
    Config as RTCConfig, DS3231, InterruptControl, Oscillator, SquareWaveFrequency,
    TimeRepresentation,
};

fn ones(num: u32) -> u32 {
    num % 10
}

fn tens(num: u32) -> u32 {
    num / 10
}

// Interrupt driven
#[embassy_executor::task]
pub async fn rtc_event(i2c_bus: &'static SharedI2CBusMutex, mut exti: ExtiInput<'static>) {
    let shared_i2c_device = I2cDevice::new(i2c_bus);
    let mut rtc = DS3231::new(shared_i2c_device, 0x68);
    //let mut rtc = Ds323x::new_ds3231(shared_i2c_device);

    let datetime_start = NaiveDate::from_ymd_opt(2025, 8, 10)
        .unwrap()
        .and_hms_opt(1, 32, 33)
        .unwrap();

    // Using middle-income man's interrupt method by
    // using 1hz square wave output as interrupt.
    // Because ds3231 interrupt output is open drain and
    // I don't know how to make it work with stm32f3-discovery inputs.
    // Perhaps it needs an external power source with pull-up resistor.
    let rtc_config: RTCConfig = RTCConfig {
        time_representation: TimeRepresentation::TwentyFourHour,
        square_wave_frequency: SquareWaveFrequency::Hz1,
        interrupt_control: InterruptControl::SquareWave,
        battery_backed_square_wave: false,
        oscillator_enable: Oscillator::Enabled,
    };
    match rtc.configure(&rtc_config).await {
        Ok(()) => {
            info!("RTC: Configuration successful.");
        }
        Err(_) => {
            error!("RTC ERROR: Configuration failed! Check I2C connections.");
        }
    }

    match rtc.set_datetime(&datetime_start).await {
        Ok(()) => {
            info!("RTC: Datetime set successfully.");
        }
        Err(_) => {
            error!("RTC ERROR: Could not set datetime!");
        }
    }

    loop {
        exti.wait_for_rising_edge().await;
        match rtc.datetime().await {
            Ok(dt) => {
                info!("*********************************");
                info!("* TTTTT  II  MM   MM  EEEEE  !! *");
                info!("*   T    II  M M M M  E      !! *");
                info!("*   T    II  M  M  M  EEE    !! *");
                info!("*   T    II  M     M  E         *");
                info!("*   T    II  M     M  EEEEE  !! *");
                info!("*********************************");
                info!("*** TIME PASSES! IT DOES! ***");
                info!(
                    "Time is not relative.. Look: {}{}:{}{}:{}{} {}-{}{}-{}{}",
                    tens(dt.hour()),
                    ones(dt.hour()),
                    tens(dt.minute()),
                    ones(dt.minute()),
                    tens(dt.second()),
                    ones(dt.second()),
                    dt.year(),
                    tens(dt.month()),
                    ones(dt.month()),
                    tens(dt.day()),
                    ones(dt.day()),
                );
            }
            Err(_) => {
                error!(
                    "RTC ERROR: Could not read datetime! Can't exist without TIME! Bailing out.."
                );
                break; // What can exist without time?..
            }
        }
    }
}
