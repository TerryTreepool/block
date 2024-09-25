
use log::{trace, error};

use near_base::NearResult;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, device::{Device_query_all, Device_info_list, Device_info}, try_decode_raw_object, try_encode_raw_object};

use crate::process::Process;

pub struct QueryAllDeviceRoutine {
    process: Process
}

impl QueryAllDeviceRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(Self{
            process,
        }))
    }

    #[inline]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for QueryAllDeviceRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("query device routine: header_meta: {header_meta}");

        let r = try_decode_raw_object!(Device_query_all, req, o, o, { header_meta.sequence() });

        let r: DataContent<Device_info_list> = match r {
            DataContent::Content(condition) => self.on_routine(header_meta, condition).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryAllDeviceRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, mut condition: Device_query_all) -> NearResult<Device_info_list> {

        Ok(Device_info_list {
            devices:    self.process()
                            .db_helper()
                            .query_all_with_param::<Device_info>(crate::p::GET_ALL_DEVICE.0, 
                                                                Device_info {
                                                                    brand_id: condition.take_brand_id(),
                                                                    product_id: condition.take_product_id(),
                                                                    ..Default::default()
                                                                })
                            .await
                            .map_err(| e | {
                                error!("{e}, sequence: {}", header_meta.sequence());
                                e
                            })?
                            .into_iter()
                            .filter(| thing | {
                                if condition.thing_id.len() > 0 {
                                    condition.thing_id().contains(&thing.device_id)
                                } else {
                                    true
                                }
                            })
                            .collect(),
            ..Default::default()
        })
    }
}
