
use log::{trace, error};

use near_base::{NearResult, builder_codec_macro::Empty, };
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, try_decode_raw_object, };

use crate::process::Process;

pub struct RemoveThingRoutine {
    process: Process,
}

impl RemoveThingRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(RemoveThingRoutine{
            process
        }))
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for RemoveThingRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("RemoveThingRoutine::on_routine: header_meta={header_meta}");

        let r = try_decode_raw_object!(String, req, o, o, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content(thing_id) => 
                self.on_routine(header_meta, thing_id).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl RemoveThingRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, thing_id: String) -> NearResult<Empty> {

        self.process
            .thing_storage()
            .delete_with_prefix(&thing_id)
            .await
            .map(| _ | Empty)
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })

    }
}
