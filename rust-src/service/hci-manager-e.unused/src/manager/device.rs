
use near_base::{NearResult, NearError, ErrorCode};

use protos::device::Device_info;
use topic_util::types::brand_types::Status;

use super::manager_template::{ItemTrait, UpdateItemTrait, CheckTrait};

#[derive(Clone)]
pub struct DeviceItem {
    device: Device_info,
}

impl From<DeviceItem> for Device_info {
    fn from(value: DeviceItem) -> Self {
        value.device
    }
}

impl From<Device_info> for DeviceItem {
    fn from(device: Device_info) -> Self {
        Self{
            device,
        }
    }
}

impl ItemTrait for DeviceItem {
    fn get_item_id(&self) -> &str {
        self.device.device_id()
    }
}

impl UpdateItemTrait<DeviceItem> for DeviceItem {
    fn update_item(&mut self, new_item: DeviceItem) {
        let mut_device = &mut self.device;

        debug_assert_eq!(mut_device.brand_id(), new_item.device.brand_id());

        mut_device.set_update_time(new_item.device.update_time);
        mut_device.set_status(new_item.device.status);

    }
}

impl CheckTrait for DeviceItem {
    fn check_status(&self) -> NearResult<()> {
        self.status.try_into()
            .map_or_else(| _ |{
                let error_string = format!("{}'s status is exception, can't use it.", self.product_id());
                Err(NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, error_string))
            },| status | {
                match status {
                    Status::Eanbled => Ok(()),
                    Status::Disabled => {
                        let error_string = format!("{} brand is diabled, cann't use it.", self.brand_id());
                        Err(NearError::new(ErrorCode::NEAR_ERROR_NO_AVAILABLE, error_string))
                    }
                }
            })
    }
}

impl std::ops::Deref for DeviceItem {
    type Target = Device_info;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
