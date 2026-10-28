use embassy_time::Timer;
use embassy_usb::{Builder, driver::Driver};

use crate::{
    Module,
    orientation::descriptor::RotationReport,
    usb::hid_helpers::{HidInterfaceStorage, UsbHidComm, UsbHidInterface},
};

mod descriptor;

pub struct OrientationModule<'d, D: Driver<'d>> {
    interface: UsbHidInterface<'d, D, RotationReport, 1, 1>,
    comm: UsbHidComm<RotationReport>,
}

impl<'b, 'd, D: Driver<'d>> OrientationModule<'d, D> {
    pub fn new(builder: &'b mut Builder<'d, D>, storage: &'d mut HidInterfaceStorage<'d>) -> Self {
        let (iface, comm) = UsbHidInterface::new(builder, storage);
        Self {
            interface: iface,
            comm,
        }
    }
}

#[embassy_executor::task]
async fn fetch_readings() {
    loop {
        Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn usb_send_readings() {
    // await ``
}

pub struct OrientationModuleConfig<'b, 'd, D: Driver<'d>> {
    storage: HidInterfaceStorage<'d>,
    builder: &'b mut Builder<'d, D>, // magnetometer: tle493d::TLE493DW2B6<embassy_stm32::i2c::I2c<'static, Async, Master>, embassy_time::Delay>
}

impl<'b, 'd, D: Driver<'d>> OrientationModuleConfig<'b, 'd, D> {
    pub fn new(builder: &'b mut Builder<'d, D>) -> Self {
        Self {
            storage: Default::default(),
            builder,
        }
    }
}

impl<'d, D> Module for OrientationModule<'d, D> where D: Driver<'d> + 'd {
    // type ModuleConfig = &'d mut OrientationModuleConfig<'d, 'd, D>;

    async fn run_on(mut self, spawner: &embassy_executor::Spawner) {
        match fetch_readings() {
            Ok(token) => spawner.spawn(token),
            Err(e) => match e {
                embassy_executor::SpawnError::Busy => {
                    panic!("attempted to run module multiple times")
                }
            },
        }
        self.interface.serve(&self.comm).await;
    }
}
