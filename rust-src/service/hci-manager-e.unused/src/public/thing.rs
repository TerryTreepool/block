
use near_base::{NearResult, NearError, ErrorCode};

use dataagent_util::Helper;
use protos::device::Device_info;

pub async fn get_thing(db: &Helper, thing_id: &str) -> NearResult<Device_info> {
    let mut device = Device_info::default();

    db.query_all_with_param::<Device_info>(crate::p::GET_ALL_DEVICE.0, 
                                        Device_info {
                                            device_id: thing_id.to_owned(),
                                            ..Default::default()
                                        })
    .await?
    .get_mut(0)
    .map(| item | {
        std::mem::swap(item, &mut device);
        device
    })
    .ok_or(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found [{}] device.", thing_id)))
}
