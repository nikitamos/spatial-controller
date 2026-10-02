use embedded_hal_async::i2c::SevenBitAddress;
use embedded_interfaces::codegen::interface_objects;
use embedded_interfaces::registers::{
    i2c::codecs::OneByteRegAddrCodec,
    spi::codecs::unsupported_codec::UnsupportedCodec as UnsupportedSpiCodec,
};
type SpiCodec = UnsupportedSpiCodec<()>;

interface_objects! {
    register_defaults {
        codec_error = (),
        i2c_codec = OneByteRegAddrCodec,
        spi_codec = SpiCodec,
        size = 1
    }

    register_devices [super::TLE493DW2B6]

    register BxRegister(addr=0x00, mode=r) {
        bx_msb: i8
    }
    register ByRegister(addr=0x01, mode=r) {
        by_msb: i8
    }
    register BzRegister(addr=0x02, mode=r) {
        bz_msb: i8
    }
    register TempRegister(addr=0x03, mode=r) {
        temp_msb: u8
    }
    register Bx2Register(addr=0x04, mode=r) {
        bx_lsb: u8[4..8],
        by_lsb: u8[0..4]
    }

    #[derive(Default)]
    enum DeviceId: u8{2} {
        #[default]
        0b00 A0,
        0b01 A1,
        0b10 A2,
        0b11 A3
    }

    register Temp2Register(addr=0x05, mode=r) {
        bz_lsb: u8[0..4]
        id: DeviceId = DeviceId::A0,
        temp_lsb: u8[6..8],
    }

    register DiagRegister(addr=0x06, mode=r) {
        bus_parity: u8[7] = 1,
        fuse_parity: u8[6] = 1,
        config_parity: u8[5],
        test_mode: bool[4],
        /// `true` if ADC conversion of temperature is completed
        pd3: bool[3],
        /// `true` if ADC conversion of Bx is completed
        pd0: bool[2],
        /// Frame counter
        frm: u8[0..2]
    }

    // Registers 0x07 -- 0x0c contain wake-up thresholds and are ignored so far.

    register WakeUpRegister(addr=0x0d, mode=rw) {
        /// Wake Up mode is active? Read-only
        wa: bool[7] = false,
        /// Enables Wake Up mode. Certain conditions must be fulfilled to enable
        wu: bool[6] = false,
        xh: u8[3..6] = 0x07,
        xl: u8[0..3] = 0x07
    }

    enum TestMode : u8{2} {
        0b00 NoTest,
        0b01 VHall,
        0b10 SpinTest,
        0b11 SAT
    }

    register TModeRegister(addr=0x0e, mode=rw) {
        test: TestMode[6..8] = TestMode::NoTest,
        yh: u8[3..6],
        yl: u8[0..3]
    }

    register TPhaseRegister(addr=0x0f, mode=rw) {
        /// Depends on specific test
        phase: u8[6..8],
        zh: u8[3..6],
        zy: u8[0..3]
    }

    #[derive(Default)]
    enum TriggerOptions: u8{2} {
        #[default]
        0b00 NoTrigger,
        0b01 ReadBeforeMsb,
        _ ReadAfter0x05
    }

    #[derive(Default)]
    enum TemperatureCompensation : u8{2} {
        #[default]
        0b00 NoCompensation,
        0b01 C1,
        0b10 C2,
        0b11 C3
    }

    register ConfigRegister(addr=0x10, mode=rw) {
        disable_temp: bool[7] = false,
        /// X/Y Angular Measurement (disable Z axis)
        am: bool[6] = false,
        trig: TriggerOptions[4..6],
        /// Short-Range Sensitivity
        x2: bool[3],
        /// Magnetic Temperature Compensation
        tl_mag: TemperatureCompensation[1..3],
        /// Wake-Up and Configuration Parity
        cp: u8[0]
    }

    #[derive(Default)]
    enum ReadProtocol : u8{1} {
        #[default]
        0 TwoByte,
        1 OneByte
    }

    #[derive(Default)]
    enum PowerMode : u8{2} {
        #[default]
        0b00 LowPower,
        0b01 MasterControlled,
        0b11 Fast,
        _ Reserved
    }

    register Mod1Register(addr=0x11, mode=rw) {
        /// Fuse Parity (protects [`Mod1Register`] and [`Mod2Register`])
        fp: u8[7],
        /// Can define slave address
        iic_addr: DeviceId[5..7],
        /// I2C 1-bute or 2-byte read protocol
        pr: ReadProtocol[4],
        /// Collision avoidance
        ca: bool[3] = false,
        /// Interrupt enable
        int: bool[2] = false,
        mode: PowerMode[0..2]
    }

    #[derive(Default)]
    enum UpdateRate : u8{3} {
        #[default]
        0b000 F770Hz,
        0b001 F97Hz,
        0b010 F24Hz,
        0b011 F12Hz,
        0b100 F6Hz,
        0b101 F3Hz,
        0b110 F400mHz,
        0b111 F50mHz
    }

    register Mod2Register(addr=0x13, mode=rw) {
        prd: UpdateRate[5..8],
        _: u8[0..5]
    }

    register VerRegister(addr=0x16, mode=r) {
        _: u8[6..8],
        r#type: u8[4..6],
        hwv: u8[0..4]
    }
}

impl DeviceId {
    #[inline]
    pub fn i2c_address(&self) -> SevenBitAddress {
        match self {
            DeviceId::A0 => 0x6a,
            DeviceId::A1 => 0x44,
            DeviceId::A2 => 0xf0,
            DeviceId::A3 => 0x88,
        }
    }
}
