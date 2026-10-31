#![no_std]

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("regmap buffer is too small (expected {expected}b, actual {actual}b)")]
    InsufficientRegMapBuffer { actual: usize, expected: usize },
    #[error("unknown error in c library")]
    Unknown,
}
pub type Result<T, E = Error> = core::result::Result<T, E>;

use core::marker::PhantomData;

use embedded_hal::i2c::I2c;

use crate::{bindings::TLx493D_t, types::SensorType};

#[allow(non_camel_case_types, non_snake_case, dead_code)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod types;
pub mod i2c;

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

impl<'m, I: I2c> Tlx493d<'m, I> {
    pub fn new<S: types::SensorType>(
        typ: S,
        reg_map: &'m mut [u8],
        i2c: &'m mut i2c::I2cInterface<I>,
        vtable: &'m i2c::I2cInterfaceVTable<I>,
    ) -> Result<Self> {
        // Safety: tle outlives `reg_map`, `vtable` and `i2c` since it is enclosed
        // in `Self` which validity for `'m` is ensured by borrow checker
        let mut tle = unsafe { init_tle_struct::<S>(reg_map)? };
        unsafe { i2c::init_communication(&mut tle, i2c, vtable, typ.address_raw()) };
        Ok(Self {
            tle,
            _p: PhantomData,
        })
    }
}
