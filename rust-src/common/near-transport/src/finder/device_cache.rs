
use std::sync::RwLock;

use near_base::{device::DeviceId, DeviceObject, };

use super::{inner_device_cache::InnerDeviceCache, outer_device_cache::*};

pub struct DeviceCache {
    local_id: DeviceId,
    local: RwLock<DeviceObject>,
    outer_cacher: Option<Box<dyn OuterDeviceCache>>,
    // local cache
    inner_cacher: Option<InnerDeviceCache>,
    // local_cache: RwLock<HashMap<DeviceId, DeviceObject>>,
}

impl DeviceCache {
    pub fn new(local: DeviceObject) -> Self {
        Self {
            local_id: local.object_id().clone(),
            local: RwLock::new(local),
            outer_cacher: None,
            inner_cacher: None,
        }
    }

    pub fn set_inner(mut self) -> Self {
        self.inner_cacher = Some(InnerDeviceCache::default());
        self
    }

    pub fn set_outer(mut self, outer: Box<dyn OuterDeviceCache>) -> Self {
        self.outer_cacher = Some(outer);
        self
    }
}

impl DeviceCache {
    pub fn local(&self) -> DeviceObject {
        let local = self.local.read().unwrap();
        (&*local).clone()
    }

    #[allow(unused)]
    pub fn update_local(&self, desc: &DeviceObject) {
        let mut local = self.local.write().unwrap();
        *local = desc.clone();
    }

    pub fn add(&self, device: &DeviceObject) {
        if device.object_id().eq(&self.local_id) {
            let err_string = "device is local";
            log::warn!("{err_string}");
            return;
        }

        // 添加到内存缓存
        if let Some(inner_cacher) = self.inner_cacher.as_ref() {
            inner_cacher.add(device.clone());
        }

        if let Some(outer) = &self.outer_cacher {
            let outer = outer.clone_as_cache();
            let device = device.to_owned();
            let device_id = device.object_id().clone();
            async_std::task::spawn(async move {
                outer.add(&device_id, device).await;
            });
        }
    }

    pub async fn get(&self, id: &DeviceId) -> Option<DeviceObject> {
        if self.local_id.eq(id) {
            Some(self.local())
        } else {
            match 
                if let Some(inner_cacher) = &self.inner_cacher {
                    inner_cacher.get(id)
                } else {
                    None
                } {
                Some(device) => { Some(device) }
                None => {
                    if let Some(outer_cacher) = &self.outer_cacher {
                        outer_cacher.get(id).await
                    } else {
                        None
                    }
                }
            }
        }
    }

}

// impl DeviceCache {
//     pub fn add_sn(&self, sn: &Device) {
//         let _ = self.sn_dht_cache.write().unwrap()
//             .set(&sn.desc().object_id(), sn);
//     }

//     pub fn get_nearest_of(&self, id:& DeviceId) -> Option<Device> {
//         self.sn_dht_cache.read().unwrap()
//             .get_nearest_of(id.object_id())
//             .cloned()
//     }
// }
