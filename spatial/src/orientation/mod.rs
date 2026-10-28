use embassy_stm32::{i2c::{Master, mode::MasterMode}, mode::Async};
use embassy_time::Timer;
use embedded_hal::i2c::I2c;

use crate::Module;

pub struct OrientationModule;

#[embassy_executor::task]
async fn fetch_readings() {
    loop {
        Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn usb_send_readings() {
    loop {
        Timer::after_secs(80).await;
    }
}

pub struct OrientationModuleConfig {
    // magnetometer: tle493d::TLE493DW2B6<embassy_stm32::i2c::I2c<'static, Async, Master>, embassy_time::Delay>
}

impl Module for OrientationModule {
    type ModuleConfig = ();

    fn run_on(spawner: &embassy_executor::Spawner) {
        match fetch_readings() {
            Ok(token) => spawner.spawn(token),
            Err(e) => match e {
                embassy_executor::SpawnError::Busy => todo!(),
            },
        }
        todo!()
    }

    fn configure(config: Self::ModuleConfig) -> Self {
        Self
    }
}

