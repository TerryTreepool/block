
use log::{error, trace, info};

use near_base::NearResult;
use near_transport::{HeaderMeta, Routine, EventResult, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{try_decode_raw_object, hci::schedule::{Schedule_info, Schedule_relation_info}, DataContent, try_encode_raw_object};

use crate::process::Process;

pub struct InsertThingRelationRoutine{
    process: Process,
}

impl InsertThingRelationRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(InsertThingRelationRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for InsertThingRelationRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("update group routine: header_meta={header_meta}");

        let r = 
            try_decode_raw_object!(Schedule_info, req, o, { (o.take_schedule_id(), o.take_thing_relation()) }, { header_meta.sequence() });

        let r: DataContent<Schedule_info> = match r {
            DataContent::Content((schedule_id, relations)) => 
                self.on_routine(header_meta, schedule_id, relations).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl InsertThingRelationRoutine {

    async fn on_routine(&self, header_meta: &HeaderMeta, schedule_id: String, relations: Vec<Schedule_relation_info>) -> NearResult<Schedule_info> {

        let mut schedule = 
            self.process
                .schedule_storage()
                .load_with_prefix(&schedule_id)
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        for relation in relations {
            schedule.insert_relation(relation);
        }

        self.process
            .schedule_storage()
            .update(&schedule)
            .await
            .map(| _ | {
                info!("Successfully remove relations, sequence: {}", header_meta.sequence());
            })
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        Ok(schedule.into())
    }
}