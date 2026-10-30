//! # Status module
//! This module is responsible for collecting logs
//! and error messages as well as reporting them to the host

use embassy_time::Timer;
use embassy_usb::{
    Builder,
    class::cdc_acm::{self, CdcAcmClass},
    driver::Driver,
};
use usbd_hid::descriptor::gen_hid_descriptor;

use crate::{
    Module, ModuleBuilder, StaticStorage,
    usb::{UsbHidSink},
};
use usbd_hid::descriptor::SerializedDescriptor;

pub struct StatusModule<'d, D: Driver<'d>> {
    serial: cdc_acm::CdcAcmClass<'d, D>,
    sink: &'static UsbHidSink,
}

pub struct StatusModuleStorage<'d> {
    com_state: cdc_acm::State<'d>,
}

impl<'d> StaticStorage for StatusModuleStorage<'d> {
    const DEFAULT: Self = Self {
        com_state: cdc_acm::State::new(),
    };
}

impl<'d, D: Driver<'d>> Module for StatusModule<'d, D> {
    type Storage = StatusModuleStorage<'d>;

    async fn run_on(mut self, spawner: &embassy_executor::Spawner) {
        defmt::info!("status starting");
        spawner.spawn(periodic().expect("task is tried to run twice"));
        let _ = &mut self.serial;
        let _ = self.sink;
    }
}

#[embassy_executor::task]
async fn periodic() {
    loop {
        Timer::after_millis(500).await;
        // defmt::info!("periodic task");
    }
}

pub struct StatusModuleBuilder<'b, 'd, D: Driver<'d>> {
    builder: &'b mut Builder<'d, D>,
    sink: &'static UsbHidSink,
}

impl<'b, 'd, D: Driver<'d>> StatusModuleBuilder<'b, 'd, D> {
    pub fn new(builder: &'b mut Builder<'d, D>, sink: &'static UsbHidSink) -> Self {
        Self { builder, sink }
    }
}

impl<'b, 'd, D: Driver<'d>> ModuleBuilder for StatusModuleBuilder<'b, 'd, D> {
    type Target = StatusModule<'d, D>;

    fn build(self, storage: &'static mut StatusModuleStorage<'d>) -> Self::Target {
        let serial = CdcAcmClass::new(self.builder, &mut storage.com_state, 16);
        StatusModule {
            serial,
            sink: self.sink,
        }
    }
}
