
use log::{trace, error};

use near_base::{ErrorCode, NearError, NearResult};
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object, hci::schedule::{Schedule_add, Schedule_relation_info, Schedule_info, Schedule_mode}};

use crate::{process::Process, caches::schedule::ScheduleItem};

pub struct AddScheduleRoutine {
    process: Process,
}

impl AddScheduleRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(AddScheduleRoutine{
            process
        }))
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for AddScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("AddScheduleRoutine: header_meta={header_meta}");

        let r = 
            try_decode_raw_object!(Schedule_add, req, o, { (o.take_schedule_name(), o.take_thing_relation(), o.schedule_img_idx(), o.mode()) }, { header_meta.sequence() });

        let r: DataContent<Schedule_info> = match r {
            DataContent::Content((schedule_name, relations, schedule_img_idx, schedule_mode)) => 
                self.on_routine(header_meta, schedule_name, relations, schedule_img_idx, schedule_mode).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl AddScheduleRoutine {
    async fn on_routine(&self, 
                        header_meta: &HeaderMeta, 
                        schedule_name: String, 
                        relations: Vec<Schedule_relation_info>, 
                        schedule_img_idx: u32, 
                        schedule_mode: Schedule_mode) -> NearResult<Schedule_info> {

        let mut schedule = 
            ScheduleItem::create_new(schedule_name, schedule_img_idx, schedule_mode)
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        for relation in relations {
            let _ =  schedule.insert_relation(relation);
        }

        self.process
            .schedule_storage()
            .create_new(&schedule)
            .await
            .map_err(| e | {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_ALREADY_EXIST => {
                        let error_string = format!("[{}] schedule has been exist.", schedule.schedule_name());
                        error!("{error_string}, sequence: {}", header_meta.sequence());
                        NearError::new(e.errno(), error_string)
                    }
                    _ => {
                        let error_string = format!("failed add {} schedule with err: {e}", schedule.schedule_name());
                        error!("{error_string}, sequence: {}", header_meta.sequence());
                        NearError::new(e.errno(), error_string)
                    }
                }
            })?;

        Ok(schedule.into())

    }

}