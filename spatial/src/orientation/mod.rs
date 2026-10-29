use embassy_time::Timer;

use crate::{
    Module, ModuleBuilder, StaticStorage, orientation::descriptor::RotationReport, usb::UsbHidSink,
};

pub mod descriptor;

pub struct OrientationModule {
    sink: &'static UsbHidSink,
}

#[embassy_executor::task]
async fn fetch_readings(sender: &'static UsbHidSink) {
    loop {
        Timer::after_secs(1).await;
        sender.send(RotationReport {
            rx: 1,
            ry: 2,
            rz: 3,
        });
    }
}

pub struct OrientationModuleConfig {
    sink: &'static UsbHidSink,
}

impl ModuleBuilder for OrientationModuleConfig {
    type Target = OrientationModule;
    fn build(
        self,
        storage: &'static mut <OrientationModule as Module>::Storage,
    ) -> OrientationModule {
        OrientationModule { sink: self.sink }
    }
}

impl OrientationModuleConfig {
    pub fn new(sink: &'static UsbHidSink) -> Self {
        Self { sink }
    }
}

pub struct OrientationModuleStaticStorage {
    // hid: HidInterfaceStorage<'d, RotationReport>,
}

impl StaticStorage for OrientationModuleStaticStorage {
    const DEFAULT: Self = Self {};
}

impl Module for OrientationModule {
    async fn run_on(mut self, spawner: &embassy_executor::Spawner) {
        let comm = self.sink;
        match fetch_readings(comm) {
            Ok(token) => spawner.spawn(token),
            Err(e) => match e {
                embassy_executor::SpawnError::Busy => {
                    panic!("attempted to run module multiple times")
                }
            },
        }
    }

    type Storage = OrientationModuleStaticStorage;
}
