
use near_base::{device::DeviceId, DeviceObject};

#[async_trait::async_trait]
pub trait OuterDeviceCache: Sync + Send + 'static {
    // 添加一个device并保存
    async fn add(&self, device_id: &DeviceId, device: DeviceObject);

    // 直接在本地数据查询
    async fn get(&self, device_id: &DeviceId) -> Option<DeviceObject>;

    // async fn update<U>(&self, device_id: &DeviceId, f: U) -> NearResult<()> where U: Fn(&mut DeviceObject);

    // // 本地查询，查询不到则发起查找
    // async fn search(&self, device_id: &DeviceId) -> NearResult<DeviceObject>;

    // // 校验device的owner签名是否有效
    // async fn verfiy_owner(&self, device_id: &DeviceId, device: Option<&DeviceObject>) -> NearResult<()>;

    // // 有权对象的body签名自校验
    // // async fn verfiy_own_signs(&self, object_id: &DeviceId, object: &Arc<AnyNamedObject>) -> NearResult<()>;

    fn clone_as_cache(&self) -> Box<dyn OuterDeviceCache>;
}