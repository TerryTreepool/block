
use std::sync::Arc;

use log::{trace, error, debug};

use near_base::{NearResult, builder_codec_macro::Empty};
use near_transport::{Routine, HeaderMeta, EventResult, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, try_decode_raw_object};

use crate::{process::Process, tasks::{TaskData, TaskModule}, SCHEDULE_ID, cache::thing_components::ThingComponentPtr};

use super::ScheduleId;

struct ExecuteScheduleRoutineImpl {
    process: Process,
}

#[derive(Clone)]
pub struct ExecuteScheduleRoutine(Arc<ExecuteScheduleRoutineImpl>);

impl ExecuteScheduleRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        let ret = Self(Arc::new(ExecuteScheduleRoutineImpl{ 
            process
        }));

        RoutineWrap::new(Box::new(ret))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for ExecuteScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("ExecuteScheduleRoutine::on_routine header_meta={header_meta}");

        let r = 
            try_decode_raw_object!((String, Vec<String>), req, o, o, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content((schedule_id, things)) => 
                self.on_routine(header_meta, schedule_id, things).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl ExecuteScheduleRoutine {
    pub(in self) async fn on_routine(&self, 
                                     header_meta: &HeaderMeta, 
                                     schedule_id: String,
                                     things: Vec<String>) -> NearResult<Empty> {

        let things_list = {
            let mut things_list: Vec<ThingComponentPtr> = vec![];

            for thing_id in things {
                if let Ok(thing) = self.0.process.thing_components().get_thing_by_id(&thing_id) {
                    if things_list.iter()
                                  .find(| &c | {
                                    c.thing().desc().content().owner_depend_id() == thing.thing().desc().content().owner_depend_id()
                                  })
                                  .is_none() {
                        things_list.push(thing.clone());
                    }
                }
            }

            things_list
        };

        debug!("execute {}.", things_list.len());

        let mut fut = vec![];
        for thing_ptr in things_list {
            let mut thing = thing_ptr.thing().clone();
            thing.mut_body().mut_content().mut_user_data().insert(SCHEDULE_ID.to_owned(), ScheduleId::new(&schedule_id).to_u64().to_string());

            fut.push(
                self.0.process
                    .task_manager()
                    .add_task(
                        TaskData::from((
                            TaskModule::ExecuteSchedule.into(),
                            thing
                        ))
                    )
            );
        }
                                
        for r in futures::future::join_all(fut).await {
            if let Err(e) = r {
                error!("{e}, sequence: {}", header_meta.sequence());
            }
        }

        Ok(Empty)
    }
}
