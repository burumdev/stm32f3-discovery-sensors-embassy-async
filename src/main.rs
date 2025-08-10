#![no_main]
#![no_std]

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
    i2c::{self, I2c},
    mode::{Async, Blocking},
    peripherals,
    spi::{Config as SpiConfig, Spi},
    time::Hertz,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex};

use ds323x::{
    DateTimeAccess, Ds323x, NaiveDate, SqWFreq, ic::DS3231,
    interface::I2cInterface as RtcI2cInterface,
};
use i3g4250d::I3G4250D;
use lsm303agr::{Lsm303agr, interface::I2cInterface as MagI2cInterface, mode::MagOneShot};

use static_cell::StaticCell;

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

mod tasks_lsm303agr;
use tasks_lsm303agr::*;

mod tasks_i3g4250d;
use tasks_i3g4250d::*;

mod tasks_ds3231;
use tasks_ds3231::*;

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

bind_interrupts!(struct Irqs2 {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
});

type Magnetometer = Lsm303agr<MagI2cInterface<I2c<'static, Async>>, MagOneShot>;
pub type MagnetoMutex = mutex::Mutex<CriticalSectionRawMutex, Magnetometer>;

type Gyro = I3G4250D<Spi<'static, Blocking>, Output<'static>>;
pub type GyroMutex = mutex::Mutex<CriticalSectionRawMutex, Gyro>;

type Rtc = Ds323x<RtcI2cInterface<I2c<'static, Async>>, DS3231>;
pub type RtcMutex = mutex::Mutex<CriticalSectionRawMutex, Rtc>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Does defmt work? Yes it does!\n");

    let peris = embassy_stm32::init(Default::default());

    // Magnetometer - accelerometer setup
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

    // Spawning magnetometer tasks
    spawner
        .spawn(read_mag_temperature_every_n_seconds(lsm303agr, 3))
        .unwrap();

    spawner
        .spawn(read_magnetometer_every_n_milliseconds(lsm303agr, 2048))
        .unwrap();

    spawner
        .spawn(read_accelerometer_every_n_milliseconds(lsm303agr, 777))
        .unwrap();

    // Let's first try RTC clock with a separate I2C bus and
    // see if it's working before switch to bus sharing
    let i2c_rtc = I2c::new(
        peris.I2C2,
        // SCL, SDA
        peris.PA9,
        peris.PA10,
        Irqs2,
        peris.DMA1_CH4,
        peris.DMA1_CH5,
        Hertz(100_000),
        Default::default(),
    );

    let mut rtc = Ds323x::new_ds3231(i2c_rtc);
    let datetime_start = NaiveDate::from_ymd_opt(2025, 8, 10)
        .unwrap()
        .and_hms_opt(1, 0, 0)
        .unwrap();

    // Using middle-income man's interrupt method by
    // using 1hz square wave output as interrupt.
    // Because ds3231 interrupt output is open drain and
    // I don't know how to make it work with stm32f3-discovery inputs.
    // Perhaps it needs an external power source with pull-up resistor.
    match rtc.use_int_sqw_output_as_square_wave() {
        Ok(()) => {
            info!("RTC: Using SQW pin with square wave as interrupt.");
        }
        Err(_) => {
            error!("RTC ERROR: Can't use SQW pin for interrupts!");
        }
    }

    match rtc.set_square_wave_frequency(SqWFreq::_1Hz) {
        Ok(()) => {
            info!("RTC: 1Hz frequency set for square wave output.");
        }
        Err(_) => {
            error!("RTC ERROR: Couldn't set 1Hz freq for SQW output!");
        }
    }

    match rtc.set_datetime(&datetime_start) {
        Ok(()) => {
            info!("RTC: Datetime set successfully.");
        }
        Err(_) => {
            error!("RTC ERROR: Could not set datetime!");
        }
    }

    static RTC_CELL: StaticCell<RtcMutex> = StaticCell::new();
    let rtc = RTC_CELL.init(mutex::Mutex::new(rtc));

    let rtc_int_pin = ExtiInput::new(peris.PA0, peris.EXTI0, Pull::Down);

    // Spawn RTC task
    spawner.spawn(rtc_event(rtc, rtc_int_pin)).unwrap();

    // Gyroscope setup
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = Hertz(1_000_000);

    let spi = Spi::new_blocking(peris.SPI1, peris.PA5, peris.PA7, peris.PA6, spi_config);

    let cs_pin = Output::new(peris.PE3, Level::High, Speed::Low);
    let i3g4250d = I3G4250D::new(spi, cs_pin).ok();

    // Spawning gyro tasks
    if let Some(i3g4250d) = i3g4250d {
        static I3G4250D_CELL: StaticCell<GyroMutex> = StaticCell::new();
        let i3g4250d = I3G4250D_CELL.init(mutex::Mutex::new(i3g4250d));

        spawner
            .spawn(read_gyro_temperature_every_n_seconds(i3g4250d, 5))
            .unwrap();
        spawner
            .spawn(read_gyro_every_n_milliseconds(i3g4250d, 1433))
            .unwrap();
    } else {
        warn!(
            "Could not establish SPI connection to i3g4250 gyro. But hey we can go on without it donchuwory we don't do rocket science here.."
        );
    }
}
