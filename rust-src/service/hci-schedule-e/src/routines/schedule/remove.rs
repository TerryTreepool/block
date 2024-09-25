
use log::{trace, error, info};

use near_base::{NearResult, builder_codec_macro::Empty};
use near_transport::{Routine, HeaderMeta, RoutineWrap, RoutineEventTrait, EventResult};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object};

use crate::process::Process;

pub struct RemoveSchduleRoutine {
    #[allow(unused)]
    process: Process,
}

impl RemoveSchduleRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(RemoveSchduleRoutine{
            process
        }))
    }

    #[inline]
    #[allow(unused)]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for RemoveSchduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("RemoveSchduleRoutine::on_routine header_meta={header_meta}");

        let r = try_decode_raw_object!(String, req, o, o, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content(schedule_id) => {
                self.on_routine(header_meta, schedule_id)
                    .await
                    .map(|_| Empty)
                    .into()
            },
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl RemoveSchduleRoutine {
    pub(in self) async fn on_routine(&self, header_meta: &HeaderMeta, schedule_id: String) -> NearResult<()> {

        self.process()
            .schedule_manager()
            .remove_schedule(&schedule_id)
            .await
            .map(| _ | {
                info!("Successfully remove [{schedule_id}] schedule.")
            })
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })
    }

}