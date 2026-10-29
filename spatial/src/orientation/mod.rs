
use embassy_sync::once_lock::OnceLock;
use embassy_time::Timer;
use embassy_usb::{Builder, driver::Driver};

use crate::{
    Module, ModuleBuilder, StaticStorage,
    orientation::descriptor::RotationReport,
    usb::hid_helpers::{HidInterfaceStorage, UsbHidComm, UsbHidInterface},
};

mod descriptor;

pub struct OrientationModule<'d, D: Driver<'d>> {
    interface: UsbHidInterface<
        'd,
        D,
        RotationReport,
        { core::mem::size_of::<RotationReport>() },
        { core::mem::size_of::<RotationReport>() },
    >,
}

#[embassy_executor::task]
async fn fetch_readings(sender: &'static UsbHidComm<RotationReport>) {
    loop {
        Timer::after_secs(1).await;
        sender.send(RotationReport {
            rx: 1,
            ry: 2,
            rz: 3,
        });
    }
}

#[embassy_executor::task]
async fn usb_send_readings() {
    // await ``
}

pub struct OrientationModuleConfig<'b, 'd, D: Driver<'d>> {
    builder: &'b mut Builder<'d, D>, // magnetometer: tle493d::TLE493DW2B6<embassy_stm32::i2c::I2c<'static, Async, Master>, embassy_time::Delay>
}

impl<'b, 'd, D> ModuleBuilder for OrientationModuleConfig<'b, 'd, D>
where
    D: Driver<'d> + 'd,
{
    type Target = OrientationModule<'d, D>;
    fn build(
        self,
        storage: &'static mut <OrientationModule<'d, D> as Module>::Storage,
    ) -> OrientationModule<'d, D> {
        let OrientationModuleConfig { builder } = self;
        let (hid, _comm_lock) = storage.split();
        let iface = UsbHidInterface::new(builder, hid);
        OrientationModule {
            interface: iface,
        }
    }
}

// impl<D: ModuleBuilder<OrientationModule<'static, 'static, D>

impl<'b, 'd, D: Driver<'d>> OrientationModuleConfig<'b, 'd, D> {
    pub fn new(builder: &'b mut Builder<'d, D>) -> Self {
        Self { builder }
    }
}

pub struct OrientationModuleStaticStorage<'d> {
    hid: HidInterfaceStorage<'d, RotationReport>,
    comm: OnceLock<UsbHidComm<RotationReport>>,
}

impl<'d> OrientationModuleStaticStorage<'d> {
    pub fn split(
        &'static mut self,
    ) -> (
        &'static mut HidInterfaceStorage<'d, RotationReport>,
        &'static OnceLock<UsbHidComm<RotationReport>>,
    ) {
        (&mut self.hid, &self.comm)
    }
}
impl<'d> StaticStorage for OrientationModuleStaticStorage<'d> {
    const DEFAULT: Self = Self {
        hid: HidInterfaceStorage::new(),
        comm: OnceLock::new(),
    };
}

impl<'d, D> Module for OrientationModule<'d, D>
where
    D: Driver<'d> + 'd,
{
    async fn run_on(mut self, spawner: &embassy_executor::Spawner) {
        let comm = self.interface.comm();
        match fetch_readings(comm) {
            Ok(token) => spawner.spawn(token),
            Err(e) => match e {
                embassy_executor::SpawnError::Busy => {
                    panic!("attempted to run module multiple times")
                }
            },
        }
        self.interface.serve().await;
    }

    type Storage = OrientationModuleStaticStorage<'d>;
}
