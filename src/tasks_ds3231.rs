use defmt::*;

use crate::RtcMutex;

use embassy_stm32::exti::ExtiInput;

use chrono::{Datelike, Timelike};
use ds323x::DateTimeAccess;

// Interrupt driven
#[embassy_executor::task]
pub async fn rtc_event(rtc: &'static RtcMutex, mut exti: ExtiInput<'static>) {
    loop {
        exti.wait_for_rising_edge().await;
        match rtc.lock().await.datetime() {
            Ok(dt) => {
                info!("\n*** TIME PASSES! IT DOES! ***");
                info!(
                    "Time and date: {}:{}:{} {}-{}-{}",
                    dt.hour(),
                    dt.minute(),
                    dt.second(),
                    dt.year(),
                    dt.month(),
                    dt.day(),
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
