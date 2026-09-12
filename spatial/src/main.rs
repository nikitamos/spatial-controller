#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut led = Output::new(p.PC13, Level::High, Speed::Low);
    let key = Input::new(p.PA0, Pull::Up);

    led.set_low();

    info!("setup complete");
    loop {
        if key.is_high() {
            info!("is high");
            led.set_high();
        } else {
            info!("is low");
            led.set_low();
        }

        //led.set_low();
        Timer::after_millis(300).await;
    }
}
