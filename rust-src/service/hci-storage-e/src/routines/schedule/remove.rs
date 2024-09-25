
use log::{trace, error};

use near_base::{ErrorCode, NearError, NearResult};
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object, hci::schedule::Schedule_info};

use crate::process::Process;

pub struct RemoveScheduleRoutine {
    process: Process,
}

impl RemoveScheduleRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(RemoveScheduleRoutine{
            process
        }))
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for RemoveScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("RemoveScheduleRoutine: header_meta={header_meta}");

        let r = 
            try_decode_raw_object!(String, req, o, o, { header_meta.sequence() });

        let r: DataContent<Schedule_info> = match r {
            DataContent::Content(schedule_id) => 
                self.on_routine(header_meta, schedule_id).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl RemoveScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, schedule_id: String) -> NearResult<Schedule_info> {

        let scheduel_info = 
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
                })?;

        let _ = 
            self.process
                .schedule_storage()
                .delete_with_prefix(&schedule_id)
                .await
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
                });

        Ok(scheduel_info)
    }

}