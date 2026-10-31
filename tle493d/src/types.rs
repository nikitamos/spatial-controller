use crate::bindings::{
    GEN_1_REG_MAP_SIZE, GEN_2_REG_MAP_SIZE, GEN_3_REG_MAP_SIZE, TLx493D_IICAddressType_t,
    TLx493D_SupportedSensorType_t,
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
                unsafe{ core::mem::transmute(*self as u32)}
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
