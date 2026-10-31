#![no_std]

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("regmap buffer is too small (expected {expected}b, actual {actual}b)")]
    InsufficientRegMapBuffer { actual: usize, expected: usize },
    #[error("unknown error in c library")]
    Unknown,
}
pub type Result<T, E = Error> = core::result::Result<T, E>;

use embedded_hal::i2c::I2c;

use crate::{
    bindings::{TLx493D_SupportedSensorType_t, TLx493D_t},
    types::SensorType,
};

mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod types {
    use crate::bindings::{
        self, GEN_1_REG_MAP_SIZE, GEN_2_REG_MAP_SIZE, GEN_3_REG_MAP_SIZE, TLx493D_Register_t,
        TLx493D_SupportedSensorType_t, TLx493D_t,
    };
    trait Seal {}

    #[allow(private_bounds)]
    pub trait SensorType: Seal {
        type RegMap;
        const RAW_TYPE: TLx493D_SupportedSensorType_t;
        const REG_MAP_SIZE: usize;
    }
    macro_rules! sensor_type {
        ($newtyp:ident => $typ:ident of $map:path) => {
            pub struct $newtyp;
            impl Seal for $newtyp {}
            impl SensorType for $newtyp {
                const REG_MAP_SIZE: usize = { $map as usize };
                type RegMap = [u8; Self::REG_MAP_SIZE];
                const RAW_TYPE: TLx493D_SupportedSensorType_t = TLx493D_SupportedSensorType_t::$typ;
            }
        };
    }
    sensor_type!(A1B6 => TLx493D_A1B6_e of GEN_1_REG_MAP_SIZE);
    sensor_type!(A2B6 => TLx493D_A2B6_e of GEN_2_REG_MAP_SIZE);
    sensor_type!(A2BW => TLx493D_A2BW_e of GEN_2_REG_MAP_SIZE);
    sensor_type!(P2B6 => TLx493D_P2B6_e of GEN_2_REG_MAP_SIZE);
    sensor_type!(W2B6 => TLx493D_W2B6_e of GEN_2_REG_MAP_SIZE);
    sensor_type!(W2BW => TLx493D_W2BW_e of GEN_2_REG_MAP_SIZE);
    sensor_type!(P3B6 => TLx493D_P3B6_e of GEN_3_REG_MAP_SIZE);
    sensor_type!(P3I8 => TLx493D_P3I8_e of GEN_3_REG_MAP_SIZE);
}

unsafe fn init_tle_struct<T: SensorType>(reg_map: &mut [u8]) -> Result<TLx493D_t> {
    let mut dut: TLx493D_t = unsafe { core::mem::zeroed() };
    if reg_map.len() < T::REG_MAP_SIZE {
        return Err(Error::InsufficientRegMapBuffer {
            actual: reg_map.len(),
            expected: T::REG_MAP_SIZE,
        });
    }
    unsafe {
        bindings::tlx493d_init(&raw mut dut, T::RAW_TYPE, reg_map.as_mut_ptr())
            .then_some(dut)
            .ok_or(Error::Unknown)
    }
}

pub mod i2c {
    use core::slice;

    use crate::bindings::{self, TLx493D_t};
    use embedded_hal::i2c::I2c;
    struct I2cInterface<I: I2c> {
        i2c: I,
    }

    impl<I: I2c> I2cInterface<I> {
        pub const fn vtable() -> bindings::TLx493D_ComLibraryFunctions_t {
            bindings::TLx493D_ComLibraryFunctions_t {
                init: bindings::TLx493D_ComLibraryFunctions_t__bindgen_ty_1 {
                    iic_init: Some(i2c_init_iface::<I>),
                },
                deinit: bindings::TLx493D_ComLibraryFunctions_t__bindgen_ty_2 {
                    iic_deinit: Some(i2c_deinit_iface::<I>),
                },
                transfer: bindings::TLx493D_ComLibraryFunctions_t__bindgen_ty_3 {
                    iic_transfer: Some(i2c_transfer_iface::<I>),
                },
                setReadAddress: bindings::TLx493D_ComLibraryFunctions_t__bindgen_ty_4 {
                    iic_setReadAddress: Some(i2c_set_read_address_iface::<I>),
                },
            }
        }
    }

    unsafe extern "C" fn i2c_init_iface<I: I2c>(sensor: *mut TLx493D_t) -> bool {
        // resolves to TwoWire::begin()
        true
    }

    unsafe extern "C" fn i2c_deinit_iface<I: I2c>(sensor: *mut TLx493D_t) -> bool {
        // finally resolvesto TwoWire::end()
        true
    }

    unsafe extern "C" fn i2c_transfer_iface<I: I2c>(
        sensor: *mut TLx493D_t,
        tx_buffer: *mut u8,
        tx_len: u8,
        rx_buffer: *mut u8,
        rx_len: u8,
    ) -> bool {
        let sensor = unsafe { sensor.as_mut_unchecked() };
        let address = unsafe { sensor.comInterface.comLibParams.iic_params.address };
        // like wire->transfer(sensor->comInterface.comLibParams.iic_params.address >> 1, txBuffer, txLen, rxBuffer, rxLen)
        let tx_buffer = if tx_buffer.is_null() || tx_len == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(tx_buffer, tx_len as usize) }
        };
        let rx_buffer = if rx_buffer.is_null() || rx_len == 0 {
            &mut [] as &mut [u8]
        } else {
            unsafe { slice::from_raw_parts_mut(rx_buffer, rx_len as usize) }
        };
        let i2c: &mut I2cInterface<I> = unsafe {
            sensor
                .comInterface
                .comLibObj
                .iic_obj
                .cast::<I2cInterface<I>>()
                .as_mut_unchecked()
        };

        if tx_buffer.len() > 0 {
            let res = i2c.i2c.write(address, tx_buffer);
            if let Err(e) = res {
                // todo: log error
                return false;
            }
        }
        if rx_buffer.len() > 0 {
            // FIXME: the behavior may be different from original Arduino implementation
            // that probably doesn't send ST at the end of transmission.
            let res = i2c.i2c.read(address, rx_buffer);
            if let Err(e) = res {
                // todo: log result
                return false;
            }
        }
        true
    }

    unsafe extern "C" fn i2c_set_read_address_iface<I: I2c>(sensor: *mut TLx493D_t, address: u8) {}
}
