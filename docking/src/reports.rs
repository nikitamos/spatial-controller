use usbd_hid::descriptor::generator_prelude::*;
use zerocopy::{Immutable, IntoBytes, KnownLayout, Unaligned};

// #[gen_hid_descriptor(
//     (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = MULTI_AXIS_CONTROLLER) = {
//         (collection = PHYSICAL,) = {
//             (report_id=1,) = {
//                 (usage = 0x33,) = {
//                     #[item_settings(data,variable,absolute)] input
//                 };
//                 (usage = 0x34,) = {
//                     #[item_settings(data,variable,absolute)] input
//                 };
//                 (usage = 0x35,) = {
//                     #[item_settings(data,variable,absolute)] input;
//                 };
//             };
//         };
//         (collection = PHYSICAL,) = {
//             (report_id=2,) = {
//                 (usage = 0x33,) = {
//                     #[item_settings(data,variable,absolute)] input
//                 };
//                 (usage = 0x34,) = {
//                     #[item_settings(data,variable,absolute)] input
//                 };
//                 (usage = 0x35,) = {
//                     #[item_settings(data,variable,absolute)] input;
//                 };
//             };
//         };
//     }
// )]

pub trait HidReport: IntoBytes + Immutable + KnownLayout {
    const REPORT_ID: u8;
}
pub mod nav {
    use super::HidReport;
    use zerocopy::{Immutable, IntoBytes, KnownLayout, Unaligned};
    #[derive(IntoBytes, Immutable, Unaligned,KnownLayout)]
    #[repr(C, packed)]
    pub struct Rotation {
        pub rx: i16,
        pub ry: i16,
        pub rz: i16,
    }

    impl HidReport for Rotation {
        const REPORT_ID: u8 = 2;
    }

    #[repr(C, packed)]
    #[derive(IntoBytes, Immutable, Unaligned, KnownLayout)]
    pub struct Position {
        pub x: i16,
        pub y: i16,
        pub z: i16,
    }
    impl HidReport for Position {
        const REPORT_ID: u8 = 1;
    }

    #[repr(C, packed)]
    #[derive(IntoBytes, Immutable, Unaligned, KnownLayout)]
    pub struct Buttons {
        flags: u32,
    }
    impl HidReport for Buttons {
        const REPORT_ID: u8 = 3;
    }
}

#[repr(C, packed)]
#[derive(IntoBytes, Immutable, Unaligned)]
pub struct RotationCalibration {
    pub x: u8,
}

#[repr(C, packed)]
#[derive(IntoBytes, Immutable, Unaligned)]
pub struct PositionCalibration {
    pub z: u8,
}

#[gen_hid_descriptor(
    (collection=APPLICATION, usage_page=0xFF00, report_id=2) = {
        (usage_page=0xFF00,) = {
            (usage = 0x01,) = {
                #[item_settings(data,variable)] w=feature
            };
        };
    }
)]
#[derive(IntoBytes, Immutable, Unaligned)]
pub struct Status {
    pub w: u8,
}

#[derive(IntoBytes, Immutable, Unaligned, KnownLayout)]
#[repr(C, packed)]
pub struct HidReportPacket<T: IntoBytes + Immutable + KnownLayout> {
    report_id: u8,
    value: T,
}

impl<T: HidReport> From<T> for HidReportPacket<T> {
    fn from(value: T) -> HidReportPacket<T> {
        HidReportPacket {
            report_id: T::REPORT_ID,
            value,
        }
    }
}

pub const DESCRIPTOR: [u8; 86] = [
    0x05, 0x01, 0x09, 0x08, 0xA1, 0x01, // Translation report / Rapport de translation
    0xA1, 0x00, 0x85, 0x01, 0x16, 0x00, 0x80, 0x26, 0xFF, 0x7F, 0x36, 0x00, 0x80, 0x46, 0xFF, 0x7F,
    0x09, 0x30, 0x09, 0x31, 0x09, 0x32, 0x75, 0x10, 0x95, 0x03, 0x81, 0x02, 0xC0,
    // Rotation report / Rapport de rotation
    0xA1, 0x00, 0x85, 0x02, 0x16, 0x00, 0x80, 0x26, 0xFF, 0x7F, 0x36, 0x00, 0x80, 0x46, 0xFF, 0x7F,
    0x09, 0x33, 0x09, 0x34, 0x09, 0x35, 0x75, 0x10, 0x95, 0x03, 0x81, 0x02, 0xC0,
    // Button report / Rapport des boutons
    0xA1, 0x00, 0x85, 0x03, 0x15, 0x00, 0x25, 0x01, 0x75, 0x01, 0x95, 32, 0x05, 0x09, 0x19, 1, 0x29,
    32, 0x81, 0x02, 0xC0, 0xC0,
];
