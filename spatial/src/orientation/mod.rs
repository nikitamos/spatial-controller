use core::{
    f32,
    marker::PhantomData,
    sync::atomic::{AtomicBool, Ordering::Relaxed},
};
// use libm::Float;

use defmt::Debug2Format;
use docking::reports::{self, nav::{Buttons, Rotation}};
use embassy_futures::join::join;
use embassy_stm32::{exti::ExtiInput, mode::Async};
use embassy_time::Timer;
use embedded_hal::i2c::I2c;
use glam::{I16Vec3, IVec3, Vec3};
use tle493d::{
    Tlx493d,
    i2c::{I2cInterface, I2cInterfaceVTable},
    types::SensorType,
};

use crate::{Module, ModuleBuilder, StaticStorage, usb::UsbHidSink};

pub struct OrientationModule<I: I2c + 'static> {
    sink: &'static UsbHidSink,
    tle: Tlx493d<'static, I>,
    button: ExtiInput<'static, Async>
}

type Sensor = tle493d::types::W2B6;

struct VecFmt(pub Vec3);

impl defmt::Format for VecFmt {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "[");
        for (k, c) in self.0.to_array().into_iter().enumerate() {
            let i = libm::roundf(c * 100.0) as i32;
            let whole = i / 100;
            let frac = (i % 100).abs();
            defmt::write!(fmt, "{}.{=i32:02}", whole, frac);

            if k < 2 {
                defmt::write!(fmt, " ");
            }
        }
        defmt::write!(fmt, "]");
    }
}

// #[embassy_executor::task]
async fn fetch_readings<I: I2c + 'static>(
    sender: &'static UsbHidSink,
    mut sensor: Tlx493d<'static, I>,
) {
    let mut prev_angles = Vec3::ZERO;
    let mut prev_smoothed: Vec3 = Vec3::ZERO;
    let mut prev_raw = I16Vec3::ZERO;
    const ALPHA: f32 = 0.4;
    loop {
        Timer::after_millis(100).await;

        match sensor.get_magnetic_field_and_temperature() {
            Ok((field, t)) => {
                if let Ok(f) = sensor.get_raw_magnetic_field() {
                    defmt::debug!("raw={}", f);
                    // let f = I16Vec3::from(f);
                    // if (prev_raw - f).abs().max_element() < 3 {
                    //     continue;
                    // }
                    // prev_raw = f;
                }
                let b = glam::Vec3::from(field);
                if b.is_nan() {
                    continue;
                }
                let f = ALPHA * b + prev_smoothed * (1.0 - ALPHA);
                prev_smoothed = f;
                defmt::debug!(
                    "b={}, F={}, t={}, prev={}",
                    VecFmt(b),
                    VecFmt(f),
                    t,
                    VecFmt(prev_smoothed)
                );
                // defmt::debug!("F={}", &f.to_array());

                if f.length() < 0.5 {
                    continue;
                }

                let f = f.normalize();

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
                prev_angles = euler;
                if BUTTON_DOWN.load(Relaxed) {
                    sender.send(Buttons(0x80808080));
                }
            }
            Err(e) => {
                defmt::error!("Failed to perform measurement: {}", Debug2Format(&e));
            }
        }
        defmt::info!("sending report");
    }
}

use embassy_time::Duration;

async fn button_task(mut button: ExtiInput<'_, Async>, notifier: &AtomicBool) {
    loop {
        notifier.store(false, Relaxed);
        button.wait_for_falling_edge().await;

        Timer::after(Duration::from_millis(30)).await;

        while button.is_low() {
            notifier.store(true, Relaxed);
            button.wait_for_rising_edge().await;
            Timer::after(Duration::from_millis(30)).await;
        }
    }
}

pub struct OrientationModuleConfig<I: I2c + 'static> {
    sink: &'static UsbHidSink,
    tle: Tlx493d<'static, I>,
    button: ExtiInput<'static, Async>,
}

impl<'m, I: I2c + 'static> ModuleBuilder for OrientationModuleConfig<I> {
    type Target = OrientationModule<I>;
    fn build(self, storage: &'static mut <Self::Target as Module>::Storage) -> Self::Target {
        // let i2c = storage.i2c.insert(self.i2c);
        Self::Target {
            sink: self.sink,
            tle: self.tle, //Tlx493d::new(Sensor::A0, &mut storage.regmap, i2c, &storage.vtbl).unwrap(),
            button: self.button
        }
    }
}

impl<I: I2c> OrientationModuleConfig<I> {
    pub fn new(
        sink: &'static UsbHidSink,
        tle: Tlx493d<'static, I>,
        button: ExtiInput<'static, Async>,
    ) -> Self {
        Self { sink, tle, button }
    }
}

pub struct OrientationModuleStaticStorage<I: I2c> {
    _p: PhantomData<I>, // vtbl: I2cInterfaceVTable<I>,
                        // regmap: <Sensor as SensorType>::RegMap,
                        // i2c: Option<I2cInterface<I>>,
}

impl<I: I2c> StaticStorage for OrientationModuleStaticStorage<I> {
    const DEFAULT: Self = Self {
        _p: PhantomData, // vtbl: I2cInterfaceVTable::new(),
                         // regmap: [0; <Sensor as SensorType>::REG_MAP_SIZE],
                         // i2c: None,
    };
}

impl<I: I2c + 'static> Module for OrientationModule<I> {
    async fn run_on(self, _spawner: &embassy_executor::Spawner) {
        let comm = self.sink;
        join(fetch_readings(comm, self.tle), button_task(self.button, &BUTTON_DOWN)).await;
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

static BUTTON_DOWN: AtomicBool = AtomicBool::new(false);
