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
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, blocking_mutex::raw::NoopRawMutex, mutex::Mutex,
};

use i3g4250d::I3G4250D;

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

type SharedI2CBusMutex = Mutex<NoopRawMutex, I2c<'static, Async>>;

type Gyro = I3G4250D<Spi<'static, Blocking>, Output<'static>>;
pub type GyroMutex = Mutex<CriticalSectionRawMutex, Gyro>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Does defmt work? Yes it does!\n");

    let peris = embassy_stm32::init(Default::default());

    // Shared I2C Bus
    // To be shared between LSM303AGR magnetometer-accelerometer and
    // DS3231 Realtime Clock
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

    static I2C_CELL: StaticCell<SharedI2CBusMutex> = StaticCell::new();
    let shared_i2c_bus = I2C_CELL.init(Mutex::new(i2c));

    // Spawning magnetometer tasks
    spawner
        .spawn(read_mag_temperature_every_n_seconds(shared_i2c_bus, 3))
        .unwrap();

    spawner
        .spawn(read_magnetometer_every_n_milliseconds(shared_i2c_bus, 2048))
        .unwrap();

    spawner
        .spawn(read_accelerometer_every_n_milliseconds(shared_i2c_bus, 777))
        .unwrap();

    // RTC Setup
    let rtc_int_pin = ExtiInput::new(peris.PA0, peris.EXTI0, Pull::Down);

    // Spawn RTC task
    spawner
        .spawn(rtc_event(shared_i2c_bus, rtc_int_pin))
        .unwrap();

    // Gyroscope setup
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = Hertz(1_000_000);

    let spi = Spi::new_blocking(peris.SPI1, peris.PA5, peris.PA7, peris.PA6, spi_config);

    let cs_pin = Output::new(peris.PE3, Level::High, Speed::Low);
    let i3g4250d = I3G4250D::new(spi, cs_pin).ok();

    // Spawning gyro tasks
    if let Some(i3g4250d) = i3g4250d {
        static I3G4250D_CELL: StaticCell<GyroMutex> = StaticCell::new();
        let i3g4250d = I3G4250D_CELL.init(Mutex::new(i3g4250d));

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
