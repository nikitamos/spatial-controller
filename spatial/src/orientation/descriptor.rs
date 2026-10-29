use usbd_hid::descriptor::gen_hid_descriptor;
use usbd_hid::descriptor::{AsInputReport, SerializedDescriptor};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

use crate::usb::HidReportWithId;

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = MULTI_AXIS_CONTROLLER,report_id=1) = {
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
// TODO: how do I put repr(C) on it?
// #[repr(C)]
#[derive(IntoBytes, KnownLayout, Immutable, Unaligned)]
pub struct RotationReport {
    pub rx: i8,
    pub ry: i8,
    pub rz: i8,
}

impl HidReportWithId for RotationReport {
    const REPORT_ID: u8 = 1;
}
