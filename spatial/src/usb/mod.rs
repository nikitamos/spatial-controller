use core::{
    convert::Infallible,
    sync::atomic::{AtomicU8, Ordering},
};

use defmt::{Debug2Format, error, info, warn};
use embassy_stm32::{
    Peri,
    interrupt::typelevel::Binding,
    peripherals,
    usb::{self, DmPin, DpPin, Driver, Instance},
};
use embassy_time::Timer;
use embassy_usb::{
    Builder, UsbDevice,
    class::hid::{
        HidBootProtocol, HidProtocolMode, HidReaderWriter, HidSubclass, HidWriter, ReportId,
        RequestHandler,
    },
    control::OutResponse,
};
use usbd_hid::descriptor::{AsInputReport, MouseReport, SerializedDescriptor, gen_hid_descriptor};

static HID_PROTOCOL_MODE: AtomicU8 = AtomicU8::new(HidProtocolMode::Boot as u8);
pub mod hid_helpers;

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = MULTI_AXIS_CONTROLLER) = {
        // (collection = PHYSICAL, usage = POINTER) = {
        (usage_page = GENERIC_DESKTOP,) = {
            (usage = 0x33,) = {
                #[item_settings(data,variable,relative)] rx=input
            };
            (usage = 0x34,) = {
                #[item_settings(data,variable,relative)] ry=input
            };
            (usage = 0x35,) = {
                #[item_settings(data,variable,relative)] rz=input;
            };
        };
    }
)]
#[derive(Default)]
pub struct RotationReport {
    pub rx: i8,
    pub ry: i8,
    pub rz: i8,
}

pub struct UsbBuffers<
    const EP_OUT: usize = 256,
    const CONFIG: usize = 256,
    const BOS: usize = 256,
    const CTRL: usize = 64,
> {
    pub ep_out: [u8; EP_OUT],
    pub config: [u8; CONFIG],
    pub bos: [u8; BOS],
    pub ctrl: [u8; CTRL],
}

impl<const EP_OUT: usize, const CONFIG: usize, const BOS: usize, const CTRL: usize>
    UsbBuffers<EP_OUT, CONFIG, BOS, CTRL>
{
    pub const ZEROED: Self = Self {
        ep_out: [0; EP_OUT],
        config: [0; CONFIG],
        bos: [0; BOS],
        ctrl: [0; CTRL],
    };
}

impl<const EP_OUT: usize, const CONFIG: usize, const BOS: usize, const CTRL: usize> Default
    for UsbBuffers<EP_OUT, CONFIG, BOS, CTRL>
{
    fn default() -> Self {
        Self {
            ep_out: [0; EP_OUT],
            config: [0; CONFIG],
            bos: [0; BOS],
            ctrl: [0; CTRL],
        }
    }
}

pub fn init<'p, T: Instance>(
    irqs: impl Binding<T::Interrupt, usb::InterruptHandler<T>> + 'p,
    otg_fs: Peri<'p, T>,
    dp: Peri<'p, impl DpPin<T>>,
    dm: Peri<'p, impl DmPin<T>>,
    buffers: &'p mut UsbBuffers,
    request_handler: &'p mut MyRequestHandler,
    state: &'p mut embassy_usb::class::hid::State<'p>,
) -> (
    UsbDevice<'p, Driver<'p, T>>,
    impl Future<Output = Infallible>,
) {
    // Create the driver, from the HAL.
    let mut config = embassy_stm32::usb::Config::default();

    // Do not enable vbus_detection. This is a safe default that works in all boards.
    // However, if your USB device is self-powered (can stay powered on if USB is unplugged), you need
    // to enable vbus_detection to comply with the USB spec. If you enable it, the board
    // has to support it or USB won't work at all. See docs on `vbus_detection` for details.
    config.vbus_detection = false;

    let driver = Driver::new_fs(otg_fs, irqs, dp, dm, &mut buffers.ep_out, config);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Nikand");
    config.product = Some("Spatial Controller");
    config.serial_number = Some("0");
    config.composite_with_iads = false;
    config.device_class = 0x03; // HID device
    config.device_sub_class = 0; // No subclass
    config.device_protocol = 0; // No protocol

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.

    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        &mut buffers.config,
        &mut buffers.bos,
        &mut [], // no msos descriptors
        &mut buffers.ctrl,
    );

    // Create classes on the builder.
    let config = embassy_usb::class::hid::Config {
        report_descriptor: RotationReport::desc(),
        request_handler: Some(request_handler),
        poll_ms: 60,
        max_packet_size: 8,
        hid_subclass: HidSubclass::No,
        hid_boot_protocol: HidBootProtocol::None,
    };
    info!("{}", config.report_descriptor);

    let mut writer = HidWriter::<_, 20>::new(&mut builder, state, config);

    // Build the builder.
    let usb = builder.build();

    info!("usb spawned");

    // Do stuff with the class!
    let hid_fut = async move {
        let mut x_total = 0;
        let mut y_total = 0;
        let mut z_total = 0;

        let mut buffer = [0u8; core::mem::size_of::<RotationReport>()];
        loop {
            Timer::after_millis(100).await;
            const ROTATION: i8 = 6;

            let report = if x_total == 90 {
                if y_total == 90 {
                    if z_total == 90 {
                        x_total = 0;
                        y_total = 0;
                        z_total = 0;
                        RotationReport::default()
                    } else {
                        z_total += ROTATION;
                        RotationReport {
                            rz: ROTATION,
                            ..Default::default()
                        }
                    }
                } else {
                    y_total += ROTATION;
                    RotationReport {
                        ry: ROTATION,
                        ..Default::default()
                    }
                }
            } else {
                x_total += ROTATION;
                RotationReport {
                    rx: ROTATION,
                    ..Default::default()
                }
            };
            let size = match report.serialize(&mut buffer) {
                Ok(size) => size,
                Err(_) => {
                    error!("failed to serialize report: buffer overflow");
                    continue;
                }
            };

            if HID_PROTOCOL_MODE.load(Ordering::Relaxed) == HidProtocolMode::Boot as u8 {
                match writer.write(&buffer[..size]).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send boot report: {:?}", Debug2Format(&e)),
                }
            } else {
                match writer.write_serialize(&report).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send report: {:?}", Debug2Format(&e)),
                }
            }
        }
    };

    (usb, hid_fut)
}

pub struct MyRequestHandler {}

impl RequestHandler for MyRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", Debug2Format(&id));
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", Debug2Format(&id), data);
        OutResponse::Accepted
    }

    fn get_protocol(&self) -> HidProtocolMode {
        let protocol = HidProtocolMode::from(HID_PROTOCOL_MODE.load(Ordering::Relaxed));
        info!(
            "The current HID protocol mode is: {}",
            Debug2Format(&protocol)
        );
        protocol
    }

    fn set_protocol(&mut self, protocol: HidProtocolMode) -> OutResponse {
        info!(
            "Switching to HID protocol mode: {}",
            Debug2Format(&protocol)
        );
        HID_PROTOCOL_MODE.store(protocol as u8, Ordering::Relaxed);
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, dur: u32) {
        info!("Set idle rate for {:?} to {:?}", Debug2Format(&id), dur);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        info!("Get idle rate for {:?}", Debug2Format(&id));
        None
    }
}
