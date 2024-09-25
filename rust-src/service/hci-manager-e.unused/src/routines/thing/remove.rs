
use log::{trace, error};

use near_base::{NearResult, builder_codec_utils::Empty, };
use near_core::time_utils::native_now;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, device::Device_info, try_encode_raw_object, try_decode_raw_object, };
use topic_util::types::brand_types::Status;

use crate::process::Process;

pub struct RemoveDeviceRoutine {
    process: Process,
}

impl RemoveDeviceRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(RemoveDeviceRoutine{
            process
        }))
    }

    #[inline]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for RemoveDeviceRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("RemoveDeviceRoutine::on_routine: header_meta={header_meta}");

        let r = try_decode_raw_object!(String, req, o, o, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content(thing_id) => 
                self.on_routine(header_meta, thing_id).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl RemoveDeviceRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, thing_id: String) -> NearResult<Empty> {

        self.process()
            .db_helper()
            .execute_with_param(crate::p::UPDATE_DEVICE.0, 
                                &Device_info {
                                    device_id: thing_id,
                                    status: Status::Disabled.into(),
                                    update_time: native_now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    ..Default::default()
                                })
            .await
            .map(| _ | Empty)
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })
    }
}
