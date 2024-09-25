
use log::{trace, error, info};

use near_base::NearResult;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, try_decode_raw_object};
use protos::hci::schedule::{Schedule_info, Schedule_relation_list_update, schedule_relation_list_update::Schedule_relation_list_op, Schedule_relation_list};

use crate::process::Process;

pub struct UpdateRelationsRoutine {
    process: Process,
}

impl UpdateRelationsRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(UpdateRelationsRoutine{
            process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for UpdateRelationsRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("UpdateRelationsRoutine: header_meta={header_meta}");

        let r = 
            try_decode_raw_object!(Schedule_relation_list_update, req, o, { (o.take_schedule_id(), o.op(), o.take_relations()) }, { header_meta.sequence() });

        let r: DataContent<Schedule_info> = match r {
            DataContent::Content((schedule_id, op, relations)) => 
                self.on_routine(header_meta, schedule_id, op, relations).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl UpdateRelationsRoutine {

    async fn on_routine(
        &self, 
        header_meta: &HeaderMeta, 
        schedule_id: String, 
        op: Schedule_relation_list_op, 
        mut relations: Schedule_relation_list
    ) -> NearResult<Schedule_info> {

        let mut schedule = 
            self.process
                .schedule_storage()
                .load_with_prefix(&schedule_id)
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        let store = 
            match &op {
                Schedule_relation_list_op::update => {
                    for relation in relations.take_thing_relation() {
                        schedule.insert_relation(relation);
                    }

                    true
                }
                Schedule_relation_list_op::remove => {
                    for relation in relations.take_thing_relation() {
                        schedule.remove_relation(relation.thing_id());
                    }

                    true
                }
                _ => { false/* ignore */ }
            };

        if store {
            self.process
                .schedule_storage()
                .update(&schedule)
                .await
                .map(| _ | {
                    info!("Successfully update relations, sequence: {}", header_meta.sequence());
                })
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;
        }

        Ok(schedule.into())

    }
}