use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;
use embassy_usb::{
    Builder,
    class::hid::{Config, HidBootProtocol, HidReaderWriter, HidSubclass, State},
    driver::Driver,
};
use usbd_hid::descriptor::SerializedDescriptor;

use crate::{Module, ModuleBuilder, StaticStorage};

pub trait HidReportWithId: SerializedDescriptor {
    const REPORT_ID: u8;
}

/// Determines the size of the buffer used by IN endpoints
/// This corresponds to the maximal descriptor length
pub const INPUT_BUFFER_SIZE: usize = 32;
pub const OUTPUT_BUFFER_SIZE: usize = 32;
pub const REPORT_DESCRIPTOR_SIZE: usize = 256;
type InputBuffer = [u8; INPUT_BUFFER_SIZE];
type OutputBuffer = [u8; OUTPUT_BUFFER_SIZE];

pub struct UsbHidStorage<'s> {
    // handler: GenericHidRequestHandler,
    state: embassy_usb::class::hid::State<'s>,
    comm: UsbHidSink,
    report_descriptor: [u8; REPORT_DESCRIPTOR_SIZE],
}

impl<'s> StaticStorage for UsbHidStorage<'s> {
    const DEFAULT: Self = Self {
        state: State::new(),
        comm: UsbHidSink {
            tx: Signal::new(),
            rx: Signal::new(),
        },
        report_descriptor: [0; REPORT_DESCRIPTOR_SIZE],
    };
}

pub struct UsbHidProvider<'d, D: Driver<'d>, const READ_N: usize, const WRITE_N: usize> {
    rw: HidReaderWriter<'d, D, READ_N, WRITE_N>,
    comm: &'static UsbHidSink,
}

#[derive(Default)]
pub struct UsbHidSink {
    tx: Signal<CriticalSectionRawMutex, (InputBuffer, usize)>,
    rx: Signal<CriticalSectionRawMutex, OutputBuffer>,
}


impl UsbHidSink {
    /// Sends value to the host
    pub fn send<T: HidReportWithId + zerocopy::IntoBytes + zerocopy::Immutable>(&self, val: T) {
        let mut buf = InputBuffer::default();
        buf[0] = T::REPORT_ID;
        let bytes = val.as_bytes();
        if bytes.len() >= INPUT_BUFFER_SIZE - 1 {
            defmt::error!(
                "failed to serialize report {}: buffer overflow (allocated {}b)",
                core::any::type_name::<T>(),
                INPUT_BUFFER_SIZE
            );
        } else {
            let end = bytes.len()+1;
            buf[1..end].copy_from_slice(bytes);
            self.tx.signal((buf, end));
        }
    }

    async fn recv_from_device(&self) -> (InputBuffer, usize) {
        self.tx.wait().await
    }
}

impl<'d, D, const READ_N: usize, const WRITE_N: usize> UsbHidProvider<'d, D, READ_N, WRITE_N>
where
    D: Driver<'d>,
{
    fn sleep_ms(&self) -> u64 {
        100
    }
    fn poll_ms(&self) -> u64 {
        100
    }
    #[inline(always)]
    pub async fn serve(&mut self) -> ! {
        let comm = self.comm();
        loop {
            Timer::after_millis(self.sleep_ms()).await;
            let (value, size) = comm.recv_from_device().await;
            if let Err(e) = self.rw.write(&value[..size]).await {
                defmt::error!("Failed to send report: {}", e);
            }
        }
    }
    pub fn comm(&self) -> &'static UsbHidSink {
        self.comm
    }
}

pub struct UsbHidProviderBuilder<'b, 'd, D: Driver<'d>> {
    report_descriptor: [u8; REPORT_DESCRIPTOR_SIZE],
    descriptor_pointer: usize,
    builder: &'b mut Builder<'d, D>,
}

impl<'b, 'd, D: Driver<'d>> ModuleBuilder for UsbHidProviderBuilder<'b, 'd, D> {
    type Target = UsbHidProvider<'d, D, INPUT_BUFFER_SIZE, OUTPUT_BUFFER_SIZE>;

    fn build(self, storage: &'static mut <Self::Target as Module>::Storage) -> Self::Target {
        storage.report_descriptor = self.report_descriptor;
        let rw = HidReaderWriter::new(
            self.builder,
            &mut storage.state,
            Config {
                report_descriptor: &storage.report_descriptor[..self.descriptor_pointer], // TODO
                request_handler: None,
                poll_ms: 100,
                max_packet_size: INPUT_BUFFER_SIZE.max(OUTPUT_BUFFER_SIZE) as u16,
                hid_subclass: HidSubclass::No,
                hid_boot_protocol: HidBootProtocol::None,
            },
        );

        UsbHidProvider {
            rw,
            comm: &storage.comm,
        }
    }
}

impl<'b, 'd, D: Driver<'d>> UsbHidProviderBuilder<'b, 'd, D> {
    pub const fn new(builder: &'b mut Builder<'d, D>) -> Self {
        Self {
            report_descriptor: [0; REPORT_DESCRIPTOR_SIZE],
            descriptor_pointer: 0,
            builder,
        }
    }
    pub fn add_report_descriptor(mut self, descriptor: &[u8]) -> Self {
        let end = self.descriptor_pointer + descriptor.len();
        if end > REPORT_DESCRIPTOR_SIZE {
            panic!("report descriptor buffer is too small");
        }
        self.report_descriptor[self.descriptor_pointer..end].copy_from_slice(descriptor);
        self.descriptor_pointer = end;
        self
    }
}

impl<'d, D: Driver<'d>, const READ_N: usize, const WRITE_N: usize> Module
    for UsbHidProvider<'d, D, READ_N, WRITE_N>
{
    type Storage = UsbHidStorage<'d>;

    async fn run_on(mut self, _spawner: &embassy_executor::Spawner) {
        self.serve().await;
    }
}

// The following code is derived from the tutorial.
// It will be required in future, when we will deal with
// host-to-device transmissions.

// struct GenericHidRequestHandler {
//     mode: AtomicU8,
// }

// impl Default for GenericHidRequestHandler {
//     fn default() -> Self {
//         Self {
//             mode: AtomicU8::new(HidProtocolMode::Boot as u8),
//         }
//     }
// }

// impl RequestHandler for GenericHidRequestHandler {
//     fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
//         info!("Get report for {:?}", Debug2Format(&id));
//         None
//     }

//     fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
//         info!("Set report for {:?}: {=[u8]}", Debug2Format(&id), data);
//         OutResponse::Accepted
//     }

//     fn get_protocol(&self) -> HidProtocolMode {
//         let protocol = HidProtocolMode::from(self.mode.load(Ordering::Relaxed));
//         info!(
//             "The current HID protocol mode is: {}",
//             Debug2Format(&protocol)
//         );
//         protocol
//     }

//     fn set_protocol(&mut self, protocol: HidProtocolMode) -> OutResponse {
//         info!(
//             "Switching to HID protocol mode: {}",
//             Debug2Format(&protocol)
//         );
//         self.mode.store(protocol as u8, Ordering::Relaxed);
//         OutResponse::Accepted
//     }

//     fn set_idle_ms(&mut self, id: Option<ReportId>, dur: u32) {
//         info!("Set idle rate for {:?} to {:?}", Debug2Format(&id), dur);
//     }

//     fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
//         info!("Get idle rate for {:?}", Debug2Format(&id));
//         None
//     }
// }
