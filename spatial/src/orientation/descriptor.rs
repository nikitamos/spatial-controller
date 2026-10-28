use usbd_hid::descriptor::gen_hid_descriptor;
use usbd_hid::descriptor::{AsInputReport, SerializedDescriptor};

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
