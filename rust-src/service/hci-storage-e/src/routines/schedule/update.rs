
use log::{trace, error};

use near_base::NearResult;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::hci::schedule::{Schedule_info, Schedule_mode};
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object};
use topic_util::types::Status;

use crate::process::Process;

pub struct UpdateScheduleRoutine {
    process: Process,
}

impl UpdateScheduleRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(UpdateScheduleRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for UpdateScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("update group routine: header_meta={header_meta}");

        let r = try_decode_raw_object!(Schedule_info, req, o, o, { header_meta.sequence() });

        let r: DataContent<Schedule_info> = match r {
            DataContent::Content(schedule) => self.on_routine(header_meta, schedule).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl UpdateScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, mut new_schedule: Schedule_info) -> NearResult<Schedule_info> {

        let new_schedule_status = if new_schedule.status == 0 {
            None
        } else { 
            Some(
                Status::try_from(new_schedule.status)
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })?
            )
        };

        let schedule_id = new_schedule.take_schedule_id();
        let schedule_name = new_schedule.take_schedule_name();
        let schedule_relations = new_schedule.take_thing_relation();

        let mut schedule = 
            self.process
                .schedule_storage()
                .load_with_prefix(&schedule_id)
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        if !schedule_name.is_empty() {
            schedule.update_name(schedule_name);
        }

        if new_schedule.schedule_img_idx > 0 {
            schedule.update_img_index(new_schedule.schedule_img_idx);
        }

        match schedule.mode() {
            Schedule_mode::TimePeriod => {
                if new_schedule.has_timeperiod_mode() {
                    schedule.update_timeperiod_mode(new_schedule.take_timeperiod_mode());
                }
            }
            Schedule_mode::Condition => {
                if new_schedule.has_condition_mode() {
                    schedule.update_condition_mode(new_schedule.take_condition_mode());
                }
            }
            _ => { /* ignore */ }
        }

        if let Some(new_schedule_status) = new_schedule_status {
            match new_schedule_status {
                Status::Eanbled => schedule.enable(),
                Status::Disabled => schedule.disable(),
            }
        }

        for relation in schedule_relations {
            schedule.insert_relation(relation);
        }

        self.process
            .schedule_storage()
            .update(&schedule)
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        Ok(schedule.into())
    }
}