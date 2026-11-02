use core::{f32, marker::PhantomData};
// use libm::Float;

use defmt::Debug2Format;
use docking::reports::{self, nav::Rotation};
use embassy_time::Timer;
use embedded_hal::i2c::I2c;
use glam::Vec3;
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

// #[embassy_executor::task]
async fn fetch_readings<I: I2c + 'static>(
    sender: &'static UsbHidSink,
    mut sensor: Tlx493d<'static, I>,
) {
    let mut prev_angles = Vec3::ZERO;

    const ALPHA: f32 = 0.6;
    loop {
        Timer::after_millis(100).await;
        match sensor.get_magnetic_field_and_temperature() {
            Ok((field, x)) => {
                defmt::debug!("B={}, t={}", field, x);
                let f = glam::Vec3::from(field).normalize();

                let yaw = libm::atan2f(-f.x, -f.z); // radians, around +Y
                let pitch = libm::asinf(f.y.clamp(-1.0, 1.0)); // radians, around +X (clamp for FP safety)
                let roll = 0.0_f32;

                // convert to degrees
                let euler = Vec3::new(pitch, yaw, roll) * 180.0 / f32::consts::PI;
                let d = euler - prev_angles;
                sender.send(Rotation {
                    rx: d.x as i16,
                    ry: d.y as i16,
                    rz: d.z as i16,
                });
                prev_angles = euler
            }
            Err(e) => {
                defmt::error!("Failed to perform measurement: {}", Debug2Format(&e));
            }
        }
        defmt::info!("sending report");
    }
}

pub struct OrientationModuleConfig<I: I2c + 'static> {
    sink: &'static UsbHidSink,
    tle: Tlx493d<'static, I>,
}

impl<'m, I: I2c + 'static> ModuleBuilder for OrientationModuleConfig<I> {
    type Target = OrientationModule<I>;
    fn build(self, storage: &'static mut <Self::Target as Module>::Storage) -> Self::Target {
        // let i2c = storage.i2c.insert(self.i2c);
        Self::Target {
            sink: self.sink,
            tle: self.tle //Tlx493d::new(Sensor::A0, &mut storage.regmap, i2c, &storage.vtbl).unwrap(),
        }
    }
}

impl<I: I2c> OrientationModuleConfig<I> {
    pub fn new(
        sink: &'static UsbHidSink,
        tle: Tlx493d<'static, I>,
    ) -> Self {
        Self {
            sink,
            tle,
        }
    }
}

pub struct OrientationModuleStaticStorage<I: I2c> {
    _p: PhantomData<I>
    // vtbl: I2cInterfaceVTable<I>,
    // regmap: <Sensor as SensorType>::RegMap,
    // i2c: Option<I2cInterface<I>>,
}

impl<I: I2c> StaticStorage for OrientationModuleStaticStorage<I> {
    const DEFAULT: Self = Self {
        _p: PhantomData
        // vtbl: I2cInterfaceVTable::new(),
        // regmap: [0; <Sensor as SensorType>::REG_MAP_SIZE],
        // i2c: None,
    };
}

impl<I: I2c + 'static> Module for OrientationModule<I> {
    async fn run_on(self, _spawner: &embassy_executor::Spawner) {
        let comm = self.sink;
        fetch_readings(comm, self.tle).await;
        // match fetch_readings(comm, self.tle) {
        //     Ok(token) => spawner.spawn(token),
        //     Err(e) => match e {
        //         embassy_executor::SpawnError::Busy => {
        //             panic!("attempted to run module multiple times")
        //         }
        //     },
        // }
    }

    type Storage = OrientationModuleStaticStorage<I>;
}
