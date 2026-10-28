#![no_std]

pub mod encoder;
pub mod magnetometer;
pub mod orientation;
pub mod usb;

pub trait ErrorSink {}

/// Represents a vertical slice which is responsible for
/// the whole range of operation related to the same concept:
/// fetching the data from sensors, processing it, reporting errors,
/// sending the data over USB, etc.
pub trait Module {
    fn run_on(self, spawner: &embassy_executor::Spawner) -> impl core::future::Future<Output = ()>;
}
