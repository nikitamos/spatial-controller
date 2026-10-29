use embassy_stm32::i2c;

pub type Error = i2c::Error;
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub trait MagnetometerInterface {
    fn write_register(&self, addr: u8, value: &[u8]) -> impl Future<Output = ()> + Send;
    fn read_register(&self, addr: u8, buffer: &mut u8) -> impl Future<Output = ()> + Send;
    const FREQUENCY: u16;
}

pub struct Magnetometer<I: MagnetometerInterface> {
    _interface: I,
}

impl<I: MagnetometerInterface> Magnetometer<I> {
    pub fn new(_interface: I) -> Self {
        Magnetometer { _interface }
    }

    pub async fn measure(&self) -> (f32, f32, f32) {
        (0.0, 0.0, 0.0)
    }
}
