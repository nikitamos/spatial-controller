#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::i2c::I2c;
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, bind_interrupts, dma, i2c, peripherals, usb};
use panic_probe as _;

use spatial::usb::MyRequestHandler;

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    DMA1_STREAM0 => dma::InterruptHandler<peripherals::DMA1_CH0>;
    DMA1_STREAM1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(25_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV25,
            mul: PllMul::MUL336,
            divp: Some(PllPDiv::DIV2), // 25mhz / 25 * 336 / 2 = 168mhz
            divq: Some(PllQDiv::DIV7), // 25mhz / 25 * 336 / 7 = 48mhz
            divr: None,
        });
        config.rcc.mux.clk48sel = mux::Clk48sel::PLL1_Q;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.mux.clk48sel = mux::Clk48sel::PLL1_Q;
    }
    let p = embassy_stm32::init(config);
    info!("initialized");

    let i2c = I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        p.DMA1_CH1,
        p.DMA1_CH0,
        Irqs,
        i2c::Config::default(),
    );
    

    let mut buffers = Default::default();
    let mut handler = MyRequestHandler {};
    let mut state = Default::default();
    let (mut usb, hid_fut) = spatial::usb::init(
        Irqs,
        p.USB_OTG_FS,
        p.PA12,
        p.PA11,
        &mut buffers,
        &mut handler,
        &mut state,
    );

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb.run(), hid_fut).await;
}
