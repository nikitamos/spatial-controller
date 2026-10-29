#![deprecated(note="Use usb::UsbHidProvider instead")]
use core::marker::PhantomData;

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal,
};
use embassy_time::Timer;
use embassy_usb::{
    Builder,
    class::hid::{
        Config, HidBootProtocol, HidReaderWriter, HidSubclass, State,
    },
    driver::Driver,
};
use usbd_hid::descriptor::{AsInputReport, SerializedDescriptor};

#[derive(Default)]
pub struct HidInterfaceStorage<'s, T> {
    // handler: GenericHidRequestHandler,
    state: embassy_usb::class::hid::State<'s>,
    comm: UsbHidComm<T>,
}

impl<'s, T> HidInterfaceStorage<'s, T> {
    pub const fn new() -> Self {
        Self {
            state: State::new(),
            comm: UsbHidComm {
                tx: Signal::new(),
                rx: Signal::new(),
            },
        }
    }
}

pub struct UsbHidInterface<
    'd,
    D: Driver<'d>,
    T: SerializedDescriptor + 'static,
    const READ_N: usize,
    const WRITE_N: usize,
> {
    _p: PhantomData<T>,
    rw: HidReaderWriter<'d, D, READ_N, WRITE_N>,
    comm: &'static UsbHidComm<T>,
}

pub struct UsbHidComm<T> {
    tx: Signal<CriticalSectionRawMutex, T>,
    rx: Signal<CriticalSectionRawMutex, T>,
}

impl<T> Default for UsbHidComm<T> {
    fn default() -> Self {
        Self {
            tx: Default::default(),
            rx: Default::default(),
        }
    }
}

impl<T> UsbHidComm<T> {
    /// Sends value to the host
    pub fn send(&self, val: T) {
        self.tx.signal(val);
    }

    /// Waits for the value from the host
    pub async fn recv(&self) -> T {
        self.rx.wait().await
    }

    async fn recv_from_device(&self) -> T {
        self.tx.wait().await
    }

    fn send_to_device(&self, val: T) {
        self.rx.signal(val);
    }
}

impl<'d, D, T, const READ_N: usize, const WRITE_N: usize> UsbHidInterface<'d, D, T, READ_N, WRITE_N>
where
    D: Driver<'d>,
    T: SerializedDescriptor + AsInputReport,
{
    pub fn new<'b>(
        builder: &'b mut Builder<'d, D>,
        storage: &'static mut HidInterfaceStorage<'d, T>,
    ) -> Self {
        let rw: HidReaderWriter<'_, D, READ_N, WRITE_N> = HidReaderWriter::new(
            builder,
            &mut storage.state,
            Config {
                report_descriptor: T::desc(),
                request_handler: None,
                poll_ms: 100,
                max_packet_size: core::mem::size_of::<T>() as u16,
                hid_subclass: HidSubclass::No,
                hid_boot_protocol: HidBootProtocol::None,
            },
        );

        Self {
            _p: PhantomData,
            rw,
            comm: &storage.comm,
        }
    }
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
            let value = comm.recv_from_device().await;
            if let Err(e) = self.rw.write_serialize(&value).await {
                defmt::error!("Failed to send report: {}", e);
            }
        }
    } 
    pub fn comm(&self) -> &'static UsbHidComm<T> {
        self.comm
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
