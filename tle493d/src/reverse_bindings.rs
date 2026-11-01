//! This module provides functions that are required by the Infineon library
//! to function properly.

use crate::{
    Float,
    bindings::{
        TLx493D_SupportedComLibraryInterfaceType_t::{TLx493D_I2C_e, TLx493D_SPI_e},
        TLx493D_t,
    },
};

#[unsafe(no_mangle)]
unsafe extern "C" fn tlx493d_transfer(
    sensor: *mut TLx493D_t,
    tx_buffer: *mut u8,
    tx_len: u8,
    rx_buffer: *mut u8,
    rx_len: u8,
) -> bool {
    unsafe {
        let sensor = sensor.as_mut_unchecked();
        // Note: this function is a direct rewrite of the `tlx493d_transfer` function,
        // which is declared in CommunicationInterface.cpp
        match sensor.comIFType {
            TLx493D_I2C_e => {
                sensor
                    .comInterface
                    .comLibFuncs
                    .read()
                    .transfer
                    .iic_transfer
                    .unwrap_unchecked()(sensor, tx_buffer, tx_len, rx_buffer, rx_len)
            }
            TLx493D_SPI_e => {
                sensor
                    .comInterface
                    .comLibFuncs
                    .read()
                    .transfer
                    .spi_transfer
                    .unwrap_unchecked()(sensor, tx_buffer, tx_len, rx_buffer, rx_len)
            }
            _ => false,
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn tlx493d_setReadAddress(sensor: *mut TLx493D_t, address: u8) {
    unsafe {
        let sensor = sensor.as_mut_unchecked();
        // Note: this function is a direct rewrite of the `tlx493d_setReadAddress` function,
        // which is declared in CommunicationInterface.cpp
        match sensor.comIFType {
            TLx493D_I2C_e => sensor
                .comInterface
                .comLibFuncs
                .read()
                .setReadAddress
                .iic_setReadAddress
                .unwrap_unchecked()(sensor, address),
            TLx493D_SPI_e => sensor
                .comInterface
                .comLibFuncs
                .read()
                .setReadAddress
                .iic_setReadAddress
                .unwrap_unchecked()(sensor, address),
            _ => {}
        }
    }
}

// unsafe extern "C" fn round_real(x: Float) -> c_long {
//     // This function is meant to replace lround()
//     x.to_int_unchecked()
// }

pub mod logging {
    use core::ffi::{CStr, c_char};
    unsafe fn make_str<'s>(x: *const c_char) -> &'s str {
        if x.is_null() {
            ""
        } else {
            unsafe { CStr::from_ptr(x).to_str().unwrap_or("<conversion failed>") }
        }
    }
    fn id<T>(x: T) -> T {
        x
    }

    macro_rules! log_message {
        ($level:ident, $p:ident) => {
            match $level {
                0 => defmt::info!("{}", $p),
                1 => defmt::warn!("{}", $p),
                2 => defmt::error!("{}", $p),
                _ => {
                    defmt::warn!("unknown log level: {}", $level);
                    defmt::info!("{}", $p);
                }
            }
        };
    }

    macro_rules! declare_log_fn {
        ($name:ident) => {
            #[unsafe(no_mangle)]
            unsafe extern "C" fn $name(level: u8, fmt: *const c_char) {
                unsafe {
                    let s = make_str(fmt);
                    log_message!(level, s);
                }
            }
        };
        ($name:ident($conv1:ident($ty1:ty))) => {
            #[unsafe(no_mangle)]
            unsafe extern "C" fn $name(level: u8, fmt: *const c_char, a1: $ty1) {
                unsafe {
                    let s = make_str(fmt);
                    let p = Printable {
                        format: s,
                        arg1: $conv1(a1),
                        arg2: (),
                        arg3: (),
                    };
                    log_message!(level, p);
                }
            }
        };
        ($name:ident($conv1:ident($ty1:ty), $conv2:ident($ty2:ty))) => {
            #[unsafe(no_mangle)]
            unsafe extern "C" fn $name(level: u8, fmt: *const c_char, a1: $ty1, a2: $ty2) {
                unsafe {
                    let s = make_str(fmt);
                    let p = Printable {
                        format: s,
                        arg1: $conv1(a1),
                        arg2: $conv2(a2),
                        arg3: (),
                    };
                    log_message!(level, p);
                }
            }
        };
        ($name:ident($conv1:ident($ty1:ty), $conv2:ident($ty2:ty), $conv3:ident($ty3:ty))) => {
            #[unsafe(no_mangle)]
            unsafe extern "C" fn $name(
                level: u8,
                fmt: *const c_char,
                a1: $ty1,
                a2: $ty2,
                a3: $ty3,
            ) {
                unsafe {
                    let s = make_str(fmt);
                    let p = Printable {
                        format: s,
                        arg1: $conv1(a1),
                        arg2: $conv2(a2),
                        arg3: $conv3(a3),
                    };
                    log_message!(level, p);
                }
            }
        };
    }

    declare_log_fn!(logPrint);
    declare_log_fn!(logPrints(make_str(*const c_char)));
    declare_log_fn!(logPrintd(id(i32)));
    declare_log_fn!(logPrintss(make_str(*const c_char), make_str(*const c_char)));
    declare_log_fn!(
        logPrintdss(make_str(*const c_char), make_str(*const c_char), make_str(*const c_char))
    );
    declare_log_fn!(logPrintds(id(i32), make_str(*const c_char)));
    declare_log_fn!(logPrintdd(id(i32), id(i32)));

    struct Printable<'s, T1: defmt::Format, T2: defmt::Format, T3: defmt::Format> {
        format: &'s str,
        arg1: T1,
        arg2: T2,
        arg3: T3,
    }

    impl<'s, T1: defmt::Format, T2: defmt::Format, T3: defmt::Format> defmt::Format
        for Printable<'s, T1, T2, T3>
    {
        fn format(&self, fmt: defmt::Formatter) {
            defmt::write!(
                fmt,
                "{}, a1=`{}`, a2=`{}` a3=`{}`",
                self.format,
                self.arg1,
                self.arg2,
                self.arg3
            )
        }
    }
}
