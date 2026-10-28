#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::i2c::I2c;
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, dma, i2c, peripherals, usb};
use embassy_sync::once_lock::OnceLock;
use embassy_usb::Builder;
use panic_probe as _;

// use embedded_hal::Pwm;

use spatial::Module;
use spatial::orientation::{OrientationModule, OrientationModuleConfig};
use spatial::usb::UsbBuffers;
use spatial::usb::hid_helpers::HidInterfaceStorage;
use tle493d::TLE493DW2B6;
use tle493d::registers::DeviceId;

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    DMA1_STREAM0 => dma::InterruptHandler<peripherals::DMA1_CH0>;
    DMA1_STREAM1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
});

static DRIVER: OnceLock<Driver<'static, peripherals::USB_OTG_FS>> = OnceLock::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
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

    // let delay = ;
    let mut i2c_cfg = i2c::Config::default();
    i2c_cfg.frequency = Hertz::khz(100);
    // i2c_cfg.gpio_speed = embassy_stm32::gpio::Speed::Medium;
    i2c_cfg.scl_pullup = true;
    i2c_cfg.sda_pullup = true;
    let mut i2c = I2c::new(p.I2C1, p.PB6, p.PB7, p.DMA1_CH1, p.DMA1_CH0, Irqs, i2c_cfg);
    info!("i2c created");

    // let mut mag = TLE493DW2B6::new(
    //     i2c,
    //     tle493d::registers::DeviceId::A0,
    //     embassy_time::Delay,
    //     tle493d::Config {
    //         temp_compensation: tle493d::registers::TemperatureCompensation::NoCompensation,
    //         disable_temp: false,
    //         update_rate: tle493d::registers::UpdateRate::F97Hz,
    //         trigger_options: tle493d::registers::TriggerOptions::NoTrigger,
    //         short_range: false,
    //     },
    // )
    // .await
    // .inspect_err(|e| defmt::error!("error creating: {}", Debug2Format(e)))
    // .unwrap();

    // let mut mag =  TLE493DW2B6::new_unconfigured(i2c, DeviceId::A0, embassy_time::Delay).await;
    // let r = mag.try_read_raw_measurement().await;
    // defmt::info!("{}", Debug2Format(&r));

    // Timer::after_millis(100).await;
    // defmt::info!("created, awaited");
    // let x = mag.try_read_raw_measurement().await;
    // defmt::info!("mes={}", Debug2Format(&x));

    // ===== USB setup =====

    let mut buffers: UsbBuffers = Default::default();
    let mut config = embassy_stm32::usb::Config::default();
    config.vbus_detection = false;

    let mut storage = HidInterfaceStorage::default();
    let driver = Driver::new_fs(
        p.USB_OTG_FS,
        Irqs,
        p.PA12,
        p.PA11,
        &mut buffers.ep_out,
        config,
    );
    let config = make_usb_config();

    let mut usb_builder = Builder::new(
        driver,
        config,
        &mut buffers.config,
        &mut buffers.bos,
        &mut [], // no msos
        &mut buffers.ctrl,
    );

    let x = OrientationModule::new(&mut usb_builder, &mut storage);
    let mut dev = usb_builder.build();
    debug!("buffer usage: {}", dev.buffer_usage());

    join(x.run_on(&spawner), dev.run()).await;
    // x.run
}

fn make_usb_config() -> embassy_usb::Config<'static> {
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Nikand");
    config.product = Some("Spatial Controller");
    config.serial_number = Some("0");
    config.composite_with_iads = false;
    // TODO: change to true when more classes are added
    config.device_class = 0x03;
    config.device_sub_class = 0;
    config.device_protocol = 0;
    config
}

// fn make_i2c(p: Peripherals) -> impl embedded_hal::i2c::ErrorType {

// }

async fn i2c_scan(i2c: &mut impl embedded_hal_async::i2c::I2c) {
    defmt::debug!("Scanning I2C bus...");
    for address in 0x03..0x78 {
        match i2c.write(address, &[0x00]).await {
            Ok(()) => {
                defmt::debug!("Found device at address: 0x{:02X}", address);
            }
            Err(e) => {
                defmt::debug!("No I2C device at 0x{:02X}: {}", address, Debug2Format(&e));
            }
        }
    }
    defmt::debug!("Scan finished.");
}
