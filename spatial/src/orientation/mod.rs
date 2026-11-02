use docking::reports;
use embassy_time::Timer;
use embedded_hal::i2c::I2c;
use tle493d::{
    Tlx493d,
    i2c::{I2cInterface, I2cInterfaceVTable},
    types::SensorType,
};

use crate::{Module, ModuleBuilder, StaticStorage, usb::UsbHidSink};

pub struct OrientationModule<I: I2c + 'static> {
    sink: &'static UsbHidSink,
    tle: Tlx493d<'static, I>,
}

type Sensor = tle493d::types::W2B6;

#[embassy_executor::task]
async fn fetch_readings(sender: &'static UsbHidSink) {
    loop {
        Timer::after_secs(1).await;
        defmt::info!("sending report");
        sender.send(reports::nav::Position { x: 1, y: -2, z: 4 });
        // sender.send(reports::nav::Rotation {
        //     rx: -4,
        //     ry: 1,
        //     rz: 10,
        // });
    }
}

pub struct OrientationModuleConfig<I: I2c + 'static> {
    sink: &'static UsbHidSink,
    i2c: I2cInterface<I>,
}

impl<'m, I: I2c + 'static> ModuleBuilder for OrientationModuleConfig<I> {
    type Target = OrientationModule<I>;
    fn build(self, storage: &'static mut <Self::Target as Module>::Storage) -> Self::Target {
        let i2c = storage.i2c.insert(self.i2c);
        Self::Target {
            sink: self.sink,
            tle: Tlx493d::new(Sensor::A0, &mut storage.regmap, i2c, &storage.vtbl).unwrap(),
        }
    }
}

impl<I: I2c> OrientationModuleConfig<I> {
    pub fn new(sink: &'static UsbHidSink, i2c: I) -> Self {
        Self {
            sink,
            i2c: I2cInterface::new(i2c),
        }
    }
}

pub struct OrientationModuleStaticStorage<I: I2c> {
    vtbl: I2cInterfaceVTable<I>,
    regmap: <Sensor as SensorType>::RegMap,
    i2c: Option<I2cInterface<I>>,
}

impl<I: I2c> StaticStorage for OrientationModuleStaticStorage<I> {
    const DEFAULT: Self = Self {
        vtbl: I2cInterfaceVTable::new(),
        regmap: [0; <Sensor as SensorType>::REG_MAP_SIZE],
        i2c: None,
    };
}

impl<I: I2c + 'static> Module for OrientationModule<I> {
    async fn run_on(self, spawner: &embassy_executor::Spawner) {
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

    type Storage = OrientationModuleStaticStorage<I>;
}
