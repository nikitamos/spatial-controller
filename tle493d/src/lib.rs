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

use core::{ffi::CStr, marker::PhantomData};

use embedded_hal::{delay::DelayNs, i2c::I2c};

use crate::{
    bindings::{
        TLx493D_IICAddressType_t, TLx493D_MeasurementType_t, TLx493D_PowerModeType_t,
        TLx493D_SensitivityType_t, TLx493D_TriggerType_t, TLx493D_UpdateRateType_t, TLx493D_t,
    },
    types::SensorType,
};

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
    _p: PhantomData<&'m mut I>,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
/// Defines the different measurement types of the sensors.
/// You have to make sure that you are using a mode which is available for
/// your used sensor family
pub enum MeasurementType {
    /// 1st and 2nd generation
    BxByBz = TLx493D_MeasurementType_t::TLx493D_BxByBz_e as u8,
    /// 1st and 3rd generation
    BxByBzTemp = TLx493D_MeasurementType_t::TLx493D_BxByBzTemp_e as u8,

    /// 3rd generation
    VHallBias = TLx493D_MeasurementType_t::TLx493D_VHall_Bias_e as u8,
    Spintest = TLx493D_MeasurementType_t::TLx493D_Spintest_e as u8,
    /// Self-test
    SAT = TLx493D_MeasurementType_t::TLx493D_SAT_test_e as u8,
    BxTemp = TLx493D_MeasurementType_t::TLx493D_BxTemp_e as u8,
    BxBy = TLx493D_MeasurementType_t::TLx493D_BxBy_e as u8,
    BzTemp = TLx493D_MeasurementType_t::TLx493D_BzTemp_e as u8,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
/// Defines the different trigger modes of the sensors.
/// You have to make sure that you are using a type which is available for
/// your used sensor family
pub enum TriggerType {
    /// 1st and 2nd generation
    NoAdcOnRead = TLx493D_TriggerType_t::TLx493D_NO_ADC_ON_READ_e as u8,
    /// 1st and 2nd generation
    AdcOnReadBeforeFirstMsb = TLx493D_TriggerType_t::TLx493D_ADC_ON_READ_BEFORE_FIRST_MSB_e as u8,
    /// 1st and 2nd generation
    AdcOnReadAfterReg05 = TLx493D_TriggerType_t::TLx493D_ADC_ON_READ_AFTER_REG_05_e as u8,

    /// 3rd generation only
    NoTrigger = TLx493D_TriggerType_t::TLx493D_NO_TRIGGER_e as u8,
    /// 3rd generation only
    AdcOnRead = TLx493D_TriggerType_t::TLx493D_ADC_ON_READ_e as u8,
    /// 3rd generation only
    AdcOnStopCondition = TLx493D_TriggerType_t::TLx493D_ADC_ON_STOP_CONDITION_e as u8,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
/// Defines the different sensitivity ranges of the sensors.
pub enum SensitivityType {
    FullRange = TLx493D_SensitivityType_t::TLx493D_FULL_RANGE_e as u8,
    ShortRange = TLx493D_SensitivityType_t::TLx493D_SHORT_RANGE_e as u8,
    ExtraShortRange = TLx493D_SensitivityType_t::TLx493D_EXTRA_SHORT_RANGE_e as u8,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
/// Defines the possible I2C addresses of the sensors.
pub enum I2CAddressType {
    A0 = TLx493D_IICAddressType_t::TLx493D_IIC_ADDR_A0_e as u8,
    A1 = TLx493D_IICAddressType_t::TLx493D_IIC_ADDR_A1_e as u8,
    A2 = TLx493D_IICAddressType_t::TLx493D_IIC_ADDR_A2_e as u8,
    A3 = TLx493D_IICAddressType_t::TLx493D_IIC_ADDR_A3_e as u8,
    A4 = TLx493D_IICAddressType_t::TLx493D_IIC_ADDR_A4_e as u8,
    A5 = TLx493D_IICAddressType_t::TLx493D_IIC_ADDR_A5_e as u8,
    A6 = TLx493D_IICAddressType_t::TLx493D_IIC_ADDR_A6_e as u8,
    A7 = TLx493D_IICAddressType_t::TLx493D_IIC_ADDR_A7_e as u8,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
/// Defines the different power modes of the sensors.
/// The available ones are depending on the sensor variant that you are using.
pub enum PowerMode {
    PowerDown = TLx493D_PowerModeType_t::TLx493D_POWER_DOWN_MODE_e as u8,
    Fast = TLx493D_PowerModeType_t::TLx493D_FAST_MODE_e as u8,
    LowPower = TLx493D_PowerModeType_t::TLx493D_LOW_POWER_MODE_e as u8,
    UltraLowPower = TLx493D_PowerModeType_t::TLx493D_ULTRA_LOW_POWER_MODE_e as u8,
    MasterControlled = TLx493D_PowerModeType_t::TLx493D_MASTER_CONTROLLED_MODE_e as u8,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
/// Defines the different update rates of the sensors.
/// The available rates depend on the sensor variant.
pub enum UpdateRate {
    /// 2nd generation
    Fast = TLx493D_UpdateRateType_t::TLx493D_UPDATE_RATE_FAST_e as u8,
    /// 2nd generation
    Slow = TLx493D_UpdateRateType_t::TLx493D_UPDATE_RATE_SLOW_e as u8,
    /// 2nd generation
    Hz770 = TLx493D_UpdateRateType_t::TLx493D_UPDATE_RATE_770_HZ_e as u8,
    /// 2nd generation
    Hz97 = TLx493D_UpdateRateType_t::TLx493D_UPDATE_RATE_97_HZ_e as u8,
    /// 2nd generation
    Hz24 = TLx493D_UpdateRateType_t::TLx493D_UPDATE_RATE_24_HZ_e as u8,
    /// 2nd generation
    Hz12 = TLx493D_UpdateRateType_t::TLx493D_UPDATE_RATE_12_HZ_e as u8,
    /// 2nd generation
    Hz6 = TLx493D_UpdateRateType_t::TLx493D_UPDATE_RATE_6_HZ_e as u8,
    /// 2nd generation
    Hz3 = TLx493D_UpdateRateType_t::TLx493D_UPDATE_RATE_3_HZ_e as u8,
    /// 2nd generation
    Hz0_4 = TLx493D_UpdateRateType_t::TLx493D_UPDATE_RATE_0_4_HZ_e as u8,
    /// 2nd generation
    Hz0_05 = TLx493D_UpdateRateType_t::TLx493D_UPDATE_RATE_0_05_HZ_e as u8,

    /// 3rd generation
    Hz1000 = TLx493D_UpdateRateType_t::TLx493D_UPDATE_RATE_1000_HZ_e as u8,
    /// 3rd generation
    Hz125 = TLx493D_UpdateRateType_t::TLx493D_UPDATE_RATE_125_HZ_e as u8,
    /// 3rd generation
    Hz31 = TLx493D_UpdateRateType_t::TLx493D_UPDATE_RATE_31_HZ_e as u8,
    /// 3rd generation
    Hz16 = TLx493D_UpdateRateType_t::TLx493D_UPDATE_RATE_16_HZ_e as u8,
}

/// Describes the wake-up threshold values of the sensor.
/// If one of thresholds is exceeded, the sensor will trigger
/// an interrupt.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WakeUpThreshold<T: PartialOrd + PartialEq> {
    /// Lower threshold value
    pub low: T,
    /// Upper threshold value
    pub high: T,
}

/// Represents the magnetic field in millitesla (with `T` = [`Float`]) or raw magnetometer
/// units (with `T = i16`). Currently it is an alias to array, where magnetic field
/// components are laid in conventional order (Bx, By, Bz).
pub type MagneticField<T = i16> = [T; 3];

/// Alias to the floating point type used by the library.
///
/// By default, it maps to `f32` since it is more common
/// for embedded platforms to have hardware support of
/// single-precision floats.
///
/// Please note that original Infineon's C library uses
/// `double` exclusively, so some features may behave
/// oddly with `f32`. `f64` can be enabled by the
/// `use-f64` feature.
pub type Float = bindings::real;

impl<'m, I: I2c> Tlx493d<'m, I> {
    /// Initializes the TLx493D sensor.
    ///
    /// This function performs similarly to the `TLx493D<_,_,_>::begin()` from
    /// the Infineon's C++ library: it initializes the structure, populates vtable for the
    /// I2C communication interface and resets the sensor's configuration.
    /// The only difference is that it doesn't initialize the I2C bus (it is
    /// passed already in initialized state) and board support class.
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
        let mut this = Self {
            tle,
            _p: PhantomData,
        };
        this.set_default_config()?;
        Ok(this)
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

    /// The function sets the default configuration for a TLx493D sensor.
    /// It puts the sensor into the 1-Byte-Read-Mode and turns off the interrupt.
    /// This is required to guarantee the proper functionality of the sensor.
    ///
    /// **Note**: this function is called by [`Self::new`] as a part of the initialization.
    pub fn set_default_config(&mut self) -> Result<()> {
        unsafe { bindings::tlx493d_setDefaultConfig(&mut self.tle).ok_or(Error::Unknown) }
    }

    /// Sets the measurement type for the sensor
    pub fn set_measurement(&mut self, meas: MeasurementType) -> Result<()> {
        unsafe {
            bindings::tlx493d_setMeasurement(&mut self.tle, meas as TLx493D_MeasurementType_t::Type)
                .ok_or_unknown(())
        }
    }

    /// Sets the trigger type for the sensor.
    pub fn set_trigger(&mut self, trigger: TriggerType) -> Result<()> {
        unsafe {
            bindings::tlx493d_setTrigger(&mut self.tle, trigger as TLx493D_TriggerType_t::Type)
                .ok_or_unknown(())
        }
    }

    /// Sets the sensitivity range of the sensor.
    pub fn set_sensitivity(&mut self, range: SensitivityType) -> Result<()> {
        unsafe {
            bindings::tlx493d_setSensitivity(
                &mut self.tle,
                range as TLx493D_SensitivityType_t::Type,
            )
            .ok_or_unknown(())
        }
    }

    /// Sets the I2C address of the sensor.
    pub fn set_i2c_address(&mut self, addr: I2CAddressType) -> Result<()> {
        unsafe {
            bindings::tlx493d_setIICAddress(&mut self.tle, addr as TLx493D_IICAddressType_t::Type)
                .ok_or_unknown(())
        }
    }

    /// Enables collision avoidance for the sensor.
    pub fn enable_collision_avoidance(&mut self) -> Result<()> {
        unsafe { bindings::tlx493d_enableCollisionAvoidance(&mut self.tle).ok_or_unknown(()) }
    }

    /// Disables collision avoidance for the sensor.
    pub fn disable_collision_avoidance(&mut self) -> Result<()> {
        unsafe { bindings::tlx493d_disableCollisionAvoidance(&mut self.tle).ok_or_unknown(()) }
    }

    /// Enables interrupts for the sensor.
    pub fn enable_interrupt(&mut self) -> Result<()> {
        unsafe { bindings::tlx493d_enableInterrupt(&mut self.tle).ok_or_unknown(()) }
    }

    /// Disables interrupts for the sensor.
    pub fn disable_interrupt(&mut self) -> Result<()> {
        unsafe { bindings::tlx493d_disableInterrupt(&mut self.tle).ok_or_unknown(()) }
    }

    /// Sets the power mode of the sensor.
    pub fn set_power_mode(&mut self, mode: PowerMode) -> Result<()> {
        unsafe {
            bindings::tlx493d_setPowerMode(&mut self.tle, mode as TLx493D_PowerModeType_t::Type)
                .ok_or_unknown(())
        }
    }

    /// Sets the update rate of the sensor.
    pub fn set_update_rate(&mut self, rate: UpdateRate) -> Result<()> {
        unsafe {
            bindings::tlx493d_setUpdateRate(&mut self.tle, rate as TLx493D_UpdateRateType_t::Type)
                .ok_or_unknown(())
        }
    }

    /// Checks whether the sensor currently holds valid measurement data.
    pub fn has_valid_data(&mut self) -> bool {
        unsafe { bindings::tlx493d_hasValidData(&mut self.tle) }
    }

    /// Checks whether the sensor is functional. It checks parity
    /// bits in the sensor's registers
    ///
    /// Returns `Ok(())` if parity is correct and `Err(_)` otherwise.
    pub fn is_functional(&mut self) -> Result<()> {
        unsafe { bindings::tlx493d_isFunctional(&mut self.tle).ok_or_unknown(()) }
    }

    /// Checks whether the sensor supports wake-up functionality.
    pub fn has_wake_up(&mut self) -> bool {
        unsafe { bindings::tlx493d_hasWakeUp(&mut self.tle) }
    }

    /// Checks whether wake-up mode is enabled.
    pub fn is_wake_up_enabled(&mut self) -> bool {
        unsafe { bindings::tlx493d_isWakeUpEnabled(&mut self.tle) }
    }

    /// Enables wake-up mode.
    pub fn enable_wake_up_mode(&mut self) -> Result<()> {
        unsafe { bindings::tlx493d_enableWakeUpMode(&mut self.tle).ok_or_unknown(()) }
    }

    /// Disables wake-up mode.
    pub fn disable_wake_up_mode(&mut self) -> Result<()> {
        unsafe { bindings::tlx493d_disableWakeUpMode(&mut self.tle).ok_or_unknown(()) }
    }

    /// Sets the wake-up thresholds as raw integer values for each axis.
    pub fn set_wake_up_thresholds_as_integer(
        &mut self,
        x: WakeUpThreshold<i16>,
        y: WakeUpThreshold<i16>,
        z: WakeUpThreshold<i16>,
    ) -> Result<()> {
        unsafe {
            bindings::tlx493d_setWakeUpThresholdsAsInteger(
                &mut self.tle,
                x.low,
                x.high,
                y.low,
                y.high,
                z.low,
                z.high,
            )
            .ok_or_unknown(())
        }
    }

    /// Sets the wake-up thresholds in mT for each axis.
    /// `temperature` is used for derating the field values.
    pub fn set_wake_up_thresholds(
        &mut self,
        temperature: Float,
        x: WakeUpThreshold<Float>,
        y: WakeUpThreshold<Float>,
        z: WakeUpThreshold<Float>,
    ) -> Result<()> {
        unsafe {
            bindings::tlx493d_setWakeUpThresholds(
                &mut self.tle,
                temperature,
                x.low,
                x.high,
                y.low,
                y.high,
                z.low,
                z.high,
            )
            .ok_or_unknown(())
        }
    }

    /// Performs a software reset of the sensor.
    /// 
    /// This function is available for generation 3 sensors only.
    pub fn software_reset(&mut self) -> Result<()> {
        unsafe { bindings::tlx493d_softwareReset(&mut self.tle).ok_or_unknown(()) }
    }

    /// Returns the sensor type as a string.
    pub fn get_type(&self) -> &str {
        let ptr = unsafe { bindings::tlx493d_getTypeAsString(&self.tle) };
        unsafe { CStr::from_ptr(ptr) }.to_str().unwrap_or("")
    }

    /// Calculates the raw magnetic field values at the given temperature.
    /// Magnetic field is supplied in millitesla.
    pub fn calculate_raw_magnetic_field_at_temperature(
        &mut self,
        raw_temp: i16,
        sens: SensitivityType,
        field: MagneticField<Float>,
    ) -> MagneticField {
        let mut raw_field = [0; 3];
        unsafe {
            bindings::tlx493d_calculateRawMagneticFieldAtTemperature(
                &mut self.tle,
                raw_temp,
                sens as TLx493D_SensitivityType_t::Type,
                field[0],
                field[1],
                field[2],
                &mut raw_field[0],
                &mut raw_field[1],
                &mut raw_field[2],
            );
        }
        raw_field
    }

    /// Performs reset of the sensor. This function is equivalent
    /// to calling [`i2c::bus_reset`] and then [`Self::set_default_config`].
    /// Result of the latter call is returned.
    ///
    /// Infineon suggests to reset the sensor after MCU reset, since
    /// it may enter an incorrect state due to power spikes or ongoing
    /// communication being aborted.
    ///
    /// The reset sequence may affect another devices connected to
    /// the same bus. In such cases consider using [`i2c::bus_reset`]
    /// with subsequent reconfiguration of each affected device.
    ///
    /// Please refer to the documentation of [`i2c::bus_reset`] for further details.
    pub fn reset<D: DelayNs>(&mut self, delay: D) -> Result<()> {
        let bus = unsafe { i2c::get_i2c::<I>(&mut self.tle) };
        i2c::bus_reset(&mut bus.i2c, delay);
        self.set_default_config()
    }
}

pub mod reverse_bindings;
