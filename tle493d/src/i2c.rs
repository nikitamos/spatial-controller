use core::{marker::PhantomData, slice};

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

impl<I: I2c> Default for I2cInterfaceVTable<I> {
    fn default() -> Self {
        Self::new()
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
    pub const fn new(i2c: I) -> Self {
        Self { i2c }
    }
    pub const fn vtable_copy(&self) -> I2cInterfaceVTable<I> {
        I2cInterfaceVTable::new()
    }
}
/// # Safety
/// This function is always safe to call, although this may change in future version
unsafe extern "C" fn i2c_init_iface<I: I2c>(_sensor: *mut TLx493D_t) -> bool {
    // resolves to TwoWire::begin()
    true
}
/// # Safety
/// This function is always safe to call, although this may change in future version
unsafe extern "C" fn i2c_deinit_iface<I: I2c>(_sensor: *mut TLx493D_t) -> bool {
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
    let address = unsafe { sensor.comInterface.comLibParams.iic_params.address } >> 1;
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

    #[cfg(feature = "defmt")]
    defmt::trace!("i2c: tx={:02x} rx_len={}", tx_buffer, rx_len);

    // TODO: consider whether I2c::transaction is appropriate here
    if !tx_buffer.is_empty() {
        let res = i2c.i2c.write(address, tx_buffer);
        if let Err(e) = res {
            #[cfg(feature = "defmt")]
            defmt::error!("i2c write failed: {}", defmt::Debug2Format(&e));
            return false;
        }
    }
    if !rx_buffer.is_empty() {
        // FIXME: the behavior may be different from original Arduino implementation
        // that probably doesn't send ST at the end of transmission.
        let res = i2c.i2c.read(address, rx_buffer);
        if let Err(e) = res {
            defmt::error!("i2c read failed: {}", defmt::Debug2Format(&e));
            return false;
        }
    }
    true
}

/// # Safety
/// This function is always safe to call, although this may change in future version
unsafe extern "C" fn i2c_set_read_address_iface<I: I2c>(_sensor: *mut TLx493D_t, _address: u8) {
}

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
