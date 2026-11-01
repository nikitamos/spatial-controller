#![no_std]

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("regmap buffer is too small (expected {expected}b, actual {actual}b)")]
    InsufficientRegMapBuffer { actual: usize, expected: usize },
    #[error("unknown error in c library")]
    Unknown,
}
pub type Result<T, E = Error> = core::result::Result<T, E>;

trait BoolExt {
    fn ok_or_unknown<T>(self, ok: T) -> Result<T>;
}

impl BoolExt for bool {
    fn ok_or_unknown<T>(self, ok: T) -> Result<T> {
        if self { Ok(ok) } else { Err(Error::Unknown) }
    }
}

use core::{
    arch::asm,
    ffi::{c_double, c_long},
    marker::PhantomData,
};

use embedded_hal::i2c::I2c;

use crate::{bindings::TLx493D_t, types::SensorType};

#[allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code
)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod i2c;
pub mod types;

/// Initializes the raw sensor structure. It is equivalent to
/// `tlx493d_init` from the official library. This function
/// does not create any bus connections nor perform actual
/// initialization of the hardware.
///
/// # Safety
/// If this function returns `Ok`, the `reg_map` *must* outlive
/// return value.
unsafe fn init_tle_struct<T: SensorType>(reg_map: &mut [u8]) -> Result<TLx493D_t> {
    let mut dut: TLx493D_t = unsafe { core::mem::zeroed() };
    if reg_map.len() < T::REG_MAP_SIZE {
        return Err(Error::InsufficientRegMapBuffer {
            actual: reg_map.len(),
            expected: T::REG_MAP_SIZE,
        });
    }
    unsafe {
        // SAFETY: this won't cause buffer overflow since register map size
        // is checked above
        bindings::tlx493d_init(&raw mut dut, T::RAW_TYPE, reg_map.as_mut_ptr())
            .then_some(dut)
            .ok_or(Error::Unknown)
    }
}

pub struct Tlx493d<'m, I: I2c> {
    tle: TLx493D_t,
    // TODO: Ensure invariance over `'m`.
    _p: PhantomData<&'m mut I>,
}

pub type MagneticField<T = i16> = [T; 3];
pub type Float = bindings::real;

impl<'m, I: I2c> Tlx493d<'m, I> {
    pub fn new<S: types::SensorType>(
        typ: S,
        reg_map: &'m mut [u8],
        i2c: &'m mut i2c::I2cInterface<I>,
        vtable: &'m i2c::I2cInterfaceVTable<I>,
    ) -> Result<Self> {
        // Safety: `tle` is outlived by `reg_map`, `vtable` and `i2c` since it is enclosed
        // in `Self` which is valid for `'m`
        let mut tle = unsafe { init_tle_struct::<S>(reg_map)? };
        unsafe { i2c::init_communication(&mut tle, i2c, vtable, typ.address_raw()) };
        Ok(Self {
            tle,
            _p: PhantomData,
        })
    }

    /// Reads registers from a TLx493D sensor.
    /// It is reading out the complete register map of the sensor and is saving the current
    /// values to the "internal" register map of the sensor object.
    pub fn read_registers(&mut self) -> Result<()> {
        unsafe { bindings::tlx493d_readRegisters(&mut self.tle).ok_or_unknown(()) }
    }

    /// Retrieves the raw temperature from a TLx493D sensor.
    /// The function is reading out the temperature registers of the sensor and is returning its value as LSB.
    pub fn get_raw_temperature(&mut self) -> Result<i16> {
        let mut res = 0;
        unsafe { bindings::tlx493d_getRawTemperature(&mut self.tle, &mut res).ok_or_unknown(res) }
    }

    /// Retrieves the raw magnetic field values from a TLx493D sensor.
    /// The function is reading the magnetic field value registers of the
    /// sensor and is returning them as LSB for the X, Y, Z-direction.
    pub fn get_raw_magnetic_field(&mut self) -> Result<MagneticField> {
        let mut field = [0; 3];
        unsafe {
            bindings::tlx493d_getRawMagneticField(
                &mut self.tle,
                &mut field[0],
                &mut field[1],
                &mut field[2],
            )
            .ok_or_unknown(field)
        }
    }

    /// Retrieves the raw magnetic field and temperature values from a TLx493D sensor.
    /// The function is reading out the magnetic field value registers, as well as the
    /// temperature value registers of the sensor and is returning them as LSB.
    pub fn get_raw_magnetic_field_and_temperature(&mut self) -> Result<(MagneticField, i16)> {
        let mut field = [0; 3];
        let mut temp: i16 = 0;
        unsafe {
            bindings::tlx493d_getRawMagneticFieldAndTemperature(
                &mut self.tle,
                &mut field[0],
                &mut field[1],
                &mut field[2],
                &mut temp,
            )
            .ok_or_unknown((field, temp))
        }
    }
    /// Retrieves the temperature from a TLx493D sensor in °C. The function is reading
    /// out the temperature registers of the sensor and converts the values into °C with
    /// the formula that is provided in the datasheet of the sensor.
    pub fn get_temperature(&mut self) -> Result<Float> {
        let mut temp = 0.0;
        unsafe { bindings::tlx493d_getTemperature(&mut self.tle, &mut temp).ok_or_unknown(temp) }
    }

    /// Retrieves the magnetic field values from a TLx493D sensor. The function is reading
    /// out the magnetic field value registers of the sensor and is converting the retrieved
    /// data into values in mT for each magnetic field axis.
    pub fn get_magnetic_field(&mut self) -> Result<MagneticField<Float>> {
        let mut field = [0.0; 3];
        unsafe {
            bindings::tlx493d_getMagneticField(
                &mut self.tle,
                &mut field[0],
                &mut field[1],
                &mut field[2],
            )
            .ok_or_unknown(field)
        }
    }

    /// The function retrieves the magnetic field and temperature readings from a TLx493D sensor.
    /// it combines the two functions [`Self::get_temperature`] and [`Self::get_magnetic_field`] in
    /// order to have one function which is providing the temperature as well as the
    /// magnetic field values of the sensor.
    pub fn get_magnetic_field_and_temperature(&mut self) -> Result<(MagneticField<Float>, Float)> {
        let mut field = [0.0; 3];
        let mut temp = 0.0;
        unsafe {
            bindings::tlx493d_getMagneticFieldAndTemperature(
                &mut self.tle,
                &mut field[0],
                &mut field[1],
                &mut field[2],
                &mut temp,
            )
            .ok_or_unknown((field, temp))
        }
    }

    // TODO: set measurement, set trigger, sensitivity, default config, i2c address, collision avoidance
}

pub mod reverse_bindings;

// #[unsafe(no_mangle)]
// unsafe extern "C" fn lround(x: c_double) -> c_long {
//     unsafe {x.to_int_unchecked()}
// }
