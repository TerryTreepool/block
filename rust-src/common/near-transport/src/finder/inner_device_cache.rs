
use std::{collections::{hash_map::Entry, HashMap}, sync::RwLock};

use near_base::{device::DeviceId, DeviceObject, };

#[derive(Default)]
pub(super) struct InnerDeviceCache {
    inner_cache: RwLock<HashMap<DeviceId, DeviceObject>>,
}

impl InnerDeviceCache {
    pub fn add(&self, device: DeviceObject) {
        // 添加到内存缓存
        match self.inner_cache.write().unwrap().entry(device.object_id().clone()) {
            Entry::Occupied(mut existed) => {
                let _ = std::mem::replace(existed.get_mut(), device);
            }
            Entry::Vacant(empty) => {
                empty.insert(device);
            }
        }
    }


    pub fn get(&self, id: &DeviceId) -> Option<DeviceObject> {
        self.inner_cache
            .read().unwrap().get(id)
            .cloned()
    }

}