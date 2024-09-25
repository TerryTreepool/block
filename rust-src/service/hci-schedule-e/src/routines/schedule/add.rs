
use log::{trace, error, info};

use near_base::{NearResult, builder_codec_macro::Empty};
use near_transport::{Routine, HeaderMeta, RoutineWrap, RoutineEventTrait, EventResult};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object, hci::schedule::Schedule_info, };

use crate::process::Process;

pub struct AddSchduleRoutine {
    #[allow(unused)]
    process: Process,
}

impl AddSchduleRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(AddSchduleRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for AddSchduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("AddSchduleRoutine::on_routine header_meta={header_meta}");

        let r = try_decode_raw_object!(Schedule_info, req, o, o, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content(schedule_info) => {
                self.on_routine(header_meta, schedule_info)
                    .await
                    .map(| _ | Empty )
                    .into()
            },
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl AddSchduleRoutine {

    pub(in self) async fn on_routine(&self, header_meta: &HeaderMeta, schedule_info: Schedule_info) -> NearResult<()> {

        let schedule_id = schedule_info.schedule_id.clone();

        self.process.schedule_manager()
            .update_schedule(schedule_info)
            .map(| _ | {
                info!("Successfully add [{schedule_id}] schedule");
            })
            .map_err(| e | {
                error!("failed add [{schedule_id}] schedule with err:{e}, sequence: {}", header_meta.sequence());
                e
            })
    }

}