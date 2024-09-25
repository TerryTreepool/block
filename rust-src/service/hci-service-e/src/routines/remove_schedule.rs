
use std::sync::Arc;

use log::{trace, debug};

use near_base::{NearResult, ObjectId, builder_codec_utils::Empty};
use near_transport::{Routine, HeaderMeta, EventResult, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, try_decode_raw_object};

use crate::process::Process;

struct RemoveScheduleRoutineImpl {
    process: Process,
}

#[derive(Clone)]
pub struct RemoveScheduleRoutine(Arc<RemoveScheduleRoutineImpl>);

impl RemoveScheduleRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        let ret = Self(Arc::new(RemoveScheduleRoutineImpl{ 
            process
        }));

        RoutineWrap::new(Box::new(ret))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for RemoveScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("RemoveScheduleRoutine::on_routine header_meta={header_meta}");

        let r = try_decode_raw_object!(ObjectId, req, o, o, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content(thing_id) => 
                self.on_routine(header_meta, thing_id).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl RemoveScheduleRoutine {
    pub(in self) async fn on_routine(&self, 
                                     header_meta: &HeaderMeta, 
                                     thing_id: ObjectId) -> NearResult<Empty> {

        self.0
            .process
            .schedule_manager()
            .remove_(&thing_id);

        debug!("{thing_id} has removed schedule, sequence: {}.", header_meta.sequence());

        Ok(Empty)
    }
}
