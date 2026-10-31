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

use crate::{
    bindings::{TLx493D_SupportedSensorType_t, TLx493D_t},
    types::SensorType,
};

mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod types {
    use crate::bindings::{
        self, GEN_1_REG_MAP_SIZE, GEN_2_REG_MAP_SIZE, GEN_3_REG_MAP_SIZE, TLx493D_IICAddressType_t,
        TLx493D_Register_t, TLx493D_SupportedSensorType_t, TLx493D_t,
    };
    trait Seal {}

    #[allow(private_bounds)]
    pub trait SensorType: Seal {
        type RegMap;
        const RAW_TYPE: TLx493D_SupportedSensorType_t;
        const REG_MAP_SIZE: usize;
        fn address_raw(&self) -> TLx493D_IICAddressType_t;
    }
    macro_rules! sensor_type {
        ($newtyp:ident {$($addr:ident: $addr_type:ident,)*} => $typ:ident of $map:path) => {
            #[repr(u32)]
            #[derive(Debug, Copy, Clone)]
            pub enum $newtyp {
                $($addr = {TLx493D_IICAddressType_t::$addr_type as u32},)*
            }
            impl Seal for $newtyp {}
            impl SensorType for $newtyp {
                const REG_MAP_SIZE: usize = { $map as usize };
                type RegMap = [u8; Self::REG_MAP_SIZE];
                const RAW_TYPE: TLx493D_SupportedSensorType_t = TLx493D_SupportedSensorType_t::$typ;
                fn address_raw(&self) -> TLx493D_IICAddressType_t {
                    // SAFETY: all $newtyp variants are also valid for TLx493D_IICAddressType_t by definition
                    unsafe{ core::mem::transmute((*self as u32))}
                }
            }
        };
    }

    // TODO: define all possible address variants of each sensor
    sensor_type!(A1B6 {A0: TLx493D_IIC_ADDR_A0_e,} => TLx493D_A1B6_e of GEN_1_REG_MAP_SIZE);
    sensor_type!(A2B6 {A0: TLx493D_IIC_ADDR_A0_e,} => TLx493D_A2B6_e of GEN_2_REG_MAP_SIZE);
    sensor_type!(A2BW {A0: TLx493D_IIC_ADDR_A0_e,} => TLx493D_A2BW_e of GEN_2_REG_MAP_SIZE);
    sensor_type!(P2B6 {A0: TLx493D_IIC_ADDR_A0_e,} => TLx493D_P2B6_e of GEN_2_REG_MAP_SIZE);
    sensor_type!(W2B6 {A0: TLx493D_IIC_ADDR_A0_e,} => TLx493D_W2B6_e of GEN_2_REG_MAP_SIZE);
    sensor_type!(W2BW {A0: TLx493D_IIC_ADDR_A0_e,} => TLx493D_W2BW_e of GEN_2_REG_MAP_SIZE);
    sensor_type!(P3B6 {A0: TLx493D_IIC_ADDR_A0_e,}=> TLx493D_P3B6_e of GEN_3_REG_MAP_SIZE);
    sensor_type!(P3I8 {A0: TLx493D_IIC_ADDR_A0_e,}=> TLx493D_P3I8_e of GEN_3_REG_MAP_SIZE);
}

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

pub mod i2c {
    use core::{marker::PhantomData, ops::Deref, slice};

    use crate::bindings::{self, TLx493D_IICAddressType_t, TLx493D_t};
    use embedded_hal::i2c::I2c;

    pub struct I2cInterface<I: I2c> {
        i2c: I,
    }

    pub struct I2cInterfaceVTable<I: I2c> {
        vtbl: bindings::TLx493D_ComLibraryFunctions_t,
        _phantom: PhantomData<I>,
    }

    impl<I: I2c> core::ops::Deref for I2cInterfaceVTable<I> {
        type Target = bindings::TLx493D_ComLibraryFunctions_t;

        fn deref(&self) -> &Self::Target {
            self.raw()
        }
    }

    impl<I: I2c> I2cInterfaceVTable<I> {
        pub const fn new() -> Self {
            Self {
                vtbl: bindings::TLx493D_ComLibraryFunctions_t {
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
                },
                _phantom: PhantomData,
            }
        }
        pub fn raw(&self) -> &bindings::TLx493D_ComLibraryFunctions_t {
            &self.vtbl
        }
    }

    impl<I: I2c> I2cInterface<I> {
        pub fn new(i2c: I) -> Self {
            Self { i2c }
        }
    }
    /// # Safety
    /// This function is always safe to call, although this may change in future version
    unsafe extern "C" fn i2c_init_iface<I: I2c>(sensor: *mut TLx493D_t) -> bool {
        // resolves to TwoWire::begin()
        true
    }
    /// # Safety
    /// This function is always safe to call, although this may change in future version
    unsafe extern "C" fn i2c_deinit_iface<I: I2c>(sensor: *mut TLx493D_t) -> bool {
        // finally resolves to TwoWire::end()
        true
    }

    /// Transfers message via I2c.
    /// # Safety
    /// The following conditions must be met:
    /// 1. `sensor` is convertible to mutable reference
    /// 2. `tx_buffer` is either null or convertible to immutable slice of length `tx_len`
    /// 3. `rx_buffer` is either null or convertible to mutable slice of length `rx_len`
    /// 4. `sensor.commInterface.comLibObj.iic_obj` is convertible to `&mut I2cInterface<I>`
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

    /// # Safety
    /// This function is always safe to call, although this may change in future version
    unsafe extern "C" fn i2c_set_read_address_iface<I: I2c>(sensor: *mut TLx493D_t, address: u8) {}

    /// Sets up the sensor's communication vtable.
    /// This is roughly equivalent to the `ifx::tlx493d::initCommunication`
    /// with `executeInit = false`.
    ///
    /// This function does not perform sensor initialization
    ///
    /// # Safety
    /// 1. `sensor` must be properly initialized
    /// 2. `comm` and `vtable` must outlive sensor
    ///
    /// # Panics
    /// The function will panic is `sensor.functions.selectIICAddress` is [`None`].
    pub(crate) unsafe fn init_communication<'s, 'c, 'v, I: I2c>(
        sensor: &'s mut TLx493D_t,
        comm: &'c mut I2cInterface<I>,
        vtable: &'v I2cInterfaceVTable<I>,
        i2c_addr_type: TLx493D_IICAddressType_t,
    ) where
        'v: 's,
        'c: 's,
    {
        sensor.comInterface.comLibObj.iic_obj = core::ptr::from_mut(comm).cast();
        // This member is never mutated by the library, so the cast_mut is safe here
        sensor.comInterface.comLibFuncs = core::ptr::from_ref(vtable.raw()).cast_mut();
        // This field seems to contain 8-bit write address
        sensor.comInterface.comLibParams.iic_params.address = unsafe {
            sensor
                .functions
                .read()
                .selectIICAddress
                .expect("sensor's vtable must be initialized")(sensor, i2c_addr_type)
        };
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
