use core::{marker::PhantomData, sync::atomic::AtomicU8, sync::atomic::Ordering};

use defmt::{Debug2Format, info};
use embassy_usb::{
    Builder,
    class::hid::{Config, HidProtocolMode, HidReaderWriter, ReportId, RequestHandler},
    control::OutResponse,
    driver::Driver,
};
use usbd_hid::descriptor::SerializedDescriptor;

#[derive(Default)]
pub struct HidInterfaceStorage<'d> {
    handler: GenericHidRequestHandler,
    state: embassy_usb::class::hid::State<'d>
}

pub struct UsbHidInterface<
    'd,
    D: Driver<'d>,
    T: SerializedDescriptor,
    const READ_N: usize,
    const WRITE_N: usize,
> {
    _p: PhantomData<T>,
    rw: HidReaderWriter<'d, D, READ_N, WRITE_N>,
}

impl<'d, D: Driver<'d>, T: SerializedDescriptor, const READ_N: usize, const WRITE_N: usize>
    UsbHidInterface<'d, D, T, READ_N, WRITE_N>
{
    pub fn new(builder: &mut Builder<'d, D>, storage: &'d mut HidInterfaceStorage<'d>) -> Self {
        let rw = HidReaderWriter::new(
            builder,
            &mut storage.state,
            Config {
                report_descriptor: T::desc(),
                request_handler: Some(&mut storage.handler),
                poll_ms: 100,
                max_packet_size: core::mem::size_of::<T>() as u16,
                hid_subclass: embassy_usb::class::hid::HidSubclass::Boot,
                hid_boot_protocol: embassy_usb::class::hid::HidBootProtocol::Mouse,
            },
        );

        Self {
            _p:  PhantomData,
            rw
        }
    }
    // pub fn 
}


struct GenericHidRequestHandler {
    mode: AtomicU8,
}

impl Default for GenericHidRequestHandler {
    fn default() -> Self {
        Self {
            mode: AtomicU8::new(HidProtocolMode::Boot as u8),
        }
    }
}

impl RequestHandler for GenericHidRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", Debug2Format(&id));
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", Debug2Format(&id), data);
        OutResponse::Accepted
    }

    fn get_protocol(&self) -> HidProtocolMode {
        let protocol = HidProtocolMode::from(self.mode.load(Ordering::Relaxed));
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
        self.mode.store(protocol as u8, Ordering::Relaxed);
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
