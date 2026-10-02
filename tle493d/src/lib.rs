#![no_std]

use embedded_hal_async::{
    delay,
    i2c::{self, SevenBitAddress},
};
use embedded_interfaces::{
    TransportError,
    i2c::I2cDeviceAsync,
    registers::{ReadableRegister, RegisterInterfaceAsync, WritableRegister},
};

pub mod registers;
use registers::DeviceId;

use crate::registers::{
    Bx2Register, Bx2RegisterUnpacked, BxRegister, ByRegister, BzRegister, ConfigRegister,
    ConfigRegisterUnpacked, DiagRegister, Mod1Register, Mod1RegisterUnpacked, Mod2Register,
    Mod2RegisterUnpacked, PowerMode, ReadProtocol, Temp2Register, Temp2RegisterUnpacked,
    TempRegister, TemperatureCompensation, TriggerOptions, UpdateRate,
};

#[derive(thiserror::Error)]
pub enum MagnetometerError<I> {
    NotReady,
    BusError(#[from] I),
}

#[derive(Debug, PartialEq, Eq)]
pub struct RawMeasurement {
    pub bx: i16,
    pub by: i16,
    pub bz: i16,
    pub temp: u16,
}

fn merge(msb: i8, lsb: u8) -> i16 {
    ((msb as i16) << 4i32) | (lsb as i16)
}

fn merge_unsigned(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 4) | (lsb as u16)
}

pub struct TLE493DW2B6<I, D>
where
    I: embedded_interfaces::registers::RegisterInterfaceAsync,
    D: embedded_hal_async::delay::DelayNs,
{
    interface: I,
    delay: D,
    address: DeviceId,
}

#[derive(Debug, Default)]
pub struct Config {
    temp_compensation: TemperatureCompensation,
    disable_temp: bool,
    update_rate: UpdateRate,
    trigger_options: TriggerOptions,
    short_range: bool,
}

impl<I, D> TLE493DW2B6<I2cDeviceAsync<I, i2c::SevenBitAddress>, D>
where
    I: i2c::I2c<i2c::SevenBitAddress> + i2c::Error,
    D: embedded_hal_async::delay::DelayNs,
{
    pub async fn new(
        interface: I,
        id: DeviceId,
        delay: D,
        config: Config,
    ) -> Result<Self, I::Error> {
        let mut this = Self {
            interface: I2cDeviceAsync::new(interface, id.i2c_address()),
            delay,
            address: id,
        };
        this.reset().await;

        this.write_register(Mod1Register::from(Mod1RegisterUnpacked {
            fp: 0, // TODO
            iic_addr: id,
            pr: ReadProtocol::OneByte,
            ca: true,
            int: false,
            mode: PowerMode::MasterControlled,
        }))
        .await?;

        this.write_register(Mod2Register::from(Mod2RegisterUnpacked {
            prd: config.update_rate,
        }))
        .await?;

        this.write_register(ConfigRegister::from(ConfigRegisterUnpacked {
            disable_temp: config.disable_temp,
            am: false,
            trig: config.trigger_options,
            x2: config.short_range,
            tl_mag: config.temp_compensation,
            cp: 0, // TODO
        }))
        .await?;

        Ok(this)
    }

    pub async fn try_read_raw_measurement(
        &mut self,
    ) -> Result<RawMeasurement, MagnetometerError<I::Error>> {
        let diag = self.read_register::<DiagRegister>().await?.unpack();
        if diag.pd0 && diag.pd3 {
            Ok(self.raw_measurement().await?)
        } else {
            Err(MagnetometerError::NotReady)
        }
    }

    async fn raw_measurement(&mut self) -> Result<RawMeasurement, I::Error> {
        let bx_msb = self.read_register::<BxRegister>().await?.read_bx_msb();
        let by_msb = self.read_register::<ByRegister>().await?.read_by_msb();
        let bz_msb = self.read_register::<BzRegister>().await?.read_bz_msb();
        let temp_msb = self.read_register::<TempRegister>().await?.read_temp_msb();
        let Bx2RegisterUnpacked { bx_lsb, by_lsb } =
            self.read_register::<Bx2Register>().await?.unpack();
        let Temp2RegisterUnpacked {
            bz_lsb, temp_lsb, ..
        } = self.read_register::<Temp2Register>().await?.unpack();
        Ok(RawMeasurement {
            bx: merge(bx_msb, bx_lsb),
            by: merge(by_msb, by_lsb),
            bz: merge(bz_msb, bz_lsb),
            temp: merge_unsigned(temp_msb, temp_lsb),
        })
    }

    async fn reset(&mut self) {
        let _ = self.i2c_write(&[0xFF]).await;
        let _ = self.i2c_write(&[0xFF]).await;
        let _ = self.i2c_write(&[0x00]).await;
        let _ = self.i2c_write(&[0x00]).await;
        self.delay.delay_us(30).await;
    }

    fn raw_i2c(&mut self) -> &mut I {
        &mut self.interface.bound_bus.interface
    }
    async fn i2c_write(&mut self, write: &[u8]) -> Result<(), I::Error> {
        let addr = self.address();
        self.raw_i2c().write(addr, write).await
    }
    fn address(&self) -> u8 {
        self.interface.bound_bus.address
    }

    async fn write_register<R: TLE493DW2B6Register + WritableRegister>(
        &mut self,
        reg: impl AsRef<R>,
    ) -> Result<(), I::Error> {
        self.interface
            .write_register(reg)
            .await
            .map_err(|e| match e {
                TransportError::Codec(c) => panic!("codec error: {c:?}"),
                TransportError::Unexpected(u) => {
                    panic!("unexpected error: {u:?}")
                }
                TransportError::Bus(_) => todo!(),
            })
    }

    async fn read_register<R: TLE493DW2B6Register + ReadableRegister>(
        &mut self,
    ) -> Result<R, I::Error> {
        self.interface.read_register().await.map_err(|e| match e {
            TransportError::Codec(c) => panic!("codec error: {c:?}"),
            TransportError::Unexpected(u) => panic!("unexpected error: {u:?}"),
            TransportError::Bus(b) => b,
        })
    }
}

pub trait TLE493DW2B6Register {}
