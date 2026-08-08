#![no_std]
#![no_main]

use core::{marker::PhantomData, ops::Deref};

use defmt::{todo, *};
use embassy_executor::Spawner;
use embassy_stm32::{
    Config, Peri,
    gpio::{AnyPin, Flex, Input, Level, Output, OutputType::PushPull, Pin, Pull, Speed},
    time::Hertz,
    timer::{
        Channel,
        low_level::CountingMode,
        simple_pwm::{PwmPin, SimplePwm},
    },
};
use embassy_time::Timer;
use embedded_hal::{Pwm, digital::v2::InputPin};
use {defmt_rtt as _, panic_probe as _};

// fn encoder_read(pin: Peri<u16>) {

// }

pub struct Encoder<'a, A: Pin, B: Pin> {
    a: Input<'a>,
    b: Input<'a>,
    _a: PhantomData<A>,
    _b: PhantomData<B>,
}

#[derive(Debug)]
pub enum EncoderReading {
    TowardsA,
    TowardsB,
    Still,
}

impl<'a, A: Pin, B: Pin> Encoder<'a, A, B> {
    const DEFAULT_PULL: Pull = Pull::Up;
    const ROTATED_LEVEL: Level = Level::Low;

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
            (Self::ROTATED_LEVEL, _) => EncoderReading::TowardsA,
            (_, Self::ROTATED_LEVEL) => EncoderReading::TowardsB,
            _ => EncoderReading::Still,
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut led = Output::new(p.PC13, Level::High, Speed::Low);
    // let pwm_pin = PwmPin::new(p.PA0, PushPull);
    // let mut pwm = SimplePwm::new(
    //     p.TIM2,
    //     Some(pwm_pin),
    //     None,
    //     None,
    //     None,
    //     Hertz(1000),
    //     CountingMode::CenterAlignedDownInterrupts,
    // );
    // pwm.set_duty(Channel::Ch1, pwm.max_duty_cycle() / 2);

    let enc1 = Encoder::new(p.PA2, p.PA0);

    loop {
        led.set_high();
        Timer::after_millis(300).await;

        led.set_low();
        Timer::after_millis(300).await;
        let reading = enc1.get_reading();
        defmt::dbg!(defmt::Debug2Format(&reading));
    }
}
