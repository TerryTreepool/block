
use near_base::{NearResult, Serialize, Deserialize, DeviceObject};

use storage::ItemTrait;

#[derive(Clone)]
pub struct DeviceObjectItem {
    device_id: String,
    device: DeviceObject,
}

impl DeviceObjectItem {
    pub fn take_device(self) -> DeviceObject {
        self.device
    }
}

impl From<DeviceObject> for DeviceObjectItem {
    fn from(device: DeviceObject) -> Self {
        Self {
            device_id: device.object_id().to_string(),
            device,
        }
    }
}

impl ItemTrait for DeviceObjectItem {
    fn id(&self) -> &str {
        &self.device_id
    }
}

impl Serialize for DeviceObjectItem {
    fn raw_capacity(&self) -> usize {
        self.device.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        self.device.serialize(buf)
    }
}

impl Deserialize for DeviceObjectItem {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (device, buf) = DeviceObject::deserialize(buf)?;

        Ok((device.into(), buf))
    }
}

impl std::ops::Deref for DeviceObjectItem {
    type Target = DeviceObject;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
