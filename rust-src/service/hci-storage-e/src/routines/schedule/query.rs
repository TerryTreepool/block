
use log::{trace, error};

use near_base::{NearResult, ErrorCode, NearError};
use near_transport::{EventResult, HeaderMeta, Routine, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object, hci::schedule::Schedule_info};

use crate::process::Process;

pub struct QueryScheduleRoutine {
    process: Process,
}

impl QueryScheduleRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(Self{
            process,
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for QueryScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("QueryScheduleRoutine: header_meta={header_meta}.");

        let r = try_decode_raw_object!(String, req, o, o, { header_meta.sequence() });

        let r: DataContent<Schedule_info> = match r {
            DataContent::Content(schedule_id) => self.on_routine(header_meta, schedule_id).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, schedule_id: String) -> NearResult<Schedule_info> {

        self.process
            .schedule_storage()
            .load_with_prefix(&schedule_id)
            .await
            .map(| schedule | schedule.into())
            .map_err(| e | {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_NOTFOUND => {
                        let error_string = format!("Not found [{}] schedule.", schedule_id);
                        error!("{error_string}, sequence: {}", header_meta.sequence());
                        NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
                    }
                    _ => {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    }
                }
            })
    }
}
