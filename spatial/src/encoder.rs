//! Abstraction for rotary encoder

use core::marker::PhantomData;
use embassy_stm32::Peri;
use embassy_stm32::gpio::Input;
use embassy_stm32::gpio::Level;
use embassy_stm32::gpio::Pin;
use embassy_stm32::gpio::Pull;

// Rotary encoder
pub struct Encoder<'a, A: Pin, B: Pin> {
    pub(crate) a: Input<'a>,
    pub(crate) b: Input<'a>,
    pub(crate) _a: PhantomData<A>,
    pub(crate) _b: PhantomData<B>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EncoderReading {
    TowardsA,
    TowardsB,
    Both,
    Still,
}

impl<'a, A: Pin, B: Pin> Encoder<'a, A, B> {
    pub(crate) const DEFAULT_PULL: Pull = Pull::Up;
    pub(crate) const ROTATED_LEVEL: Level = Level::Low;

    #[inline]
    pub fn new(a: Peri<'a, A>, b: Peri<'a, B>) -> Self {
        Self {
            a: Input::new(a, Self::DEFAULT_PULL),
            b: Input::new(b, Self::DEFAULT_PULL),
            _a: Default::default(),
            _b: Default::default(),
        }
    }

    #[inline]
    pub fn get_reading(&self) -> EncoderReading {
        // TODO: are A and B mutually exclusive?
        match (self.a.get_level(), self.b.get_level()) {
            (Self::ROTATED_LEVEL, Self::ROTATED_LEVEL) => EncoderReading::Both,
            (Self::ROTATED_LEVEL, _) => EncoderReading::TowardsA,
            (_, Self::ROTATED_LEVEL) => EncoderReading::TowardsB,
            _ => EncoderReading::Still,
        }
    }
}
