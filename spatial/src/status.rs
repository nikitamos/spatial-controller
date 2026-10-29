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
    usb::hid_helpers::{HidInterfaceStorage, UsbHidInterface},
};
use usbd_hid::descriptor::{AsInputReport, SerializedDescriptor};

pub struct StatusModule<'d, D: Driver<'d>> {
    serial: cdc_acm::CdcAcmClass<'d, D>,
    hid_endpoint: UsbHidInterface<
        'd,
        D,
        ErrorFeature,
        { core::mem::size_of::<ErrorFeature>() },
        { core::mem::size_of::<ErrorFeature>() },
    >,
}

pub struct StatusModuleStorage<'d> {
    com_state: cdc_acm::State<'d>,
    hid_state: HidInterfaceStorage<'d, ErrorFeature>
}

impl<'d> StaticStorage for StatusModuleStorage<'d> {
    const DEFAULT: Self = Self {
        com_state: cdc_acm::State::new(),
        hid_state: HidInterfaceStorage::new(),
    };
}

impl<'d, D: Driver<'d>> Module for StatusModule<'d, D> {
    type Storage = StatusModuleStorage<'d>;

    async fn run_on(mut self, spawner: &embassy_executor::Spawner) {
        defmt::info!("status starting");
        spawner.spawn(periodic().expect("task is tried to run twice"));
        self.hid_endpoint.serve().await;
    }
}

#[gen_hid_descriptor(
        (collection = APPLICATION, usage_page = VENDOR_DEFINED_START, usage = 0x01) = {
        (usage_min = X, usage_max = Y) = {
            #[item_settings(data,variable,relative)] x=input;
            #[item_settings(data,variable,relative)] y=input;
        };
    }

)]
struct ErrorFeature {
    x: u8,
    y: u16,
}

#[embassy_executor::task]
async fn periodic() {
    loop {
        Timer::after_millis(300).await;
        defmt::info!("periodic task");
    }
}

pub struct StatusModuleBuilder<'b, 'd, D: Driver<'d>> {
    builder: &'b mut Builder<'d, D>,
}

impl<'b, 'd, D: Driver<'d>> StatusModuleBuilder<'b, 'd, D> {
    pub fn new(builder: &'b mut Builder<'d, D>) -> Self {
        Self { builder }
    }
}

impl<'b, 'd, D: Driver<'d>> ModuleBuilder for StatusModuleBuilder<'b, 'd, D> {
    type Target = StatusModule<'d, D>;

    fn build(
        self,
        storage: &'static mut StatusModuleStorage<'d>,
    ) -> Self::Target {
        let serial = CdcAcmClass::new(self.builder, &mut storage.com_state, 16);
        let hid = UsbHidInterface::new(self.builder, &mut storage.hid_state);
        // storage
        StatusModule {
            serial,
            hid_endpoint: hid,
        }
    }
}
