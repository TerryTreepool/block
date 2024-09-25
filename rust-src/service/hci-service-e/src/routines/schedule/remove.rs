
use std::sync::Arc;

use log::{trace, error, warn};

use near_base::{NearResult, builder_codec_macro::Empty};
use near_transport::{Routine, HeaderMeta, EventResult, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, try_decode_raw_object};

use crate::{process::Process, tasks::{TaskData, TaskModule}, SCHEDULE_ID};

use super::ScheduleId;

struct RemoveScheduleRoutineImpl {
    process: Process,
}

#[derive(Clone)]
pub struct RemoveScheduleRoutine(Arc<RemoveScheduleRoutineImpl>);

impl RemoveScheduleRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        let ret = Self(Arc::new(RemoveScheduleRoutineImpl{ 
            process
        }));

        RoutineWrap::new(Box::new(ret))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for RemoveScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("RemoveScheduleRoutine::on_routine header_meta={header_meta}");

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

impl RemoveScheduleRoutine {
    pub(in self) async fn on_routine(&self, 
                                     header_meta: &HeaderMeta, 
                                     schedule_id: String,
                                     things: Vec<String>) -> NearResult<Empty> {

        let mut fut = vec![];

        for thing in things {

            match self.0.process.thing_components().get_thing_by_id(&thing) {
                Ok(thing) => {
                    let mut thing = thing.thing().clone();
                    thing.mut_body().mut_content().mut_user_data().insert(SCHEDULE_ID.to_owned(), ScheduleId::new(&schedule_id).to_u64().to_string());

                    fut.push(
                        self.0.process
                            .task_manager()
                            .add_task(
                                TaskData::from((
                                    TaskModule::RemoveSchedule.into(),
                                    thing
                                ))
                            )
                    );
                }
                Err(e) => {
                    warn!("{e}, sequence: {}", header_meta.sequence());
                }
            }
        }

        for r in futures::future::join_all(fut).await {
            if let Err(e) = r {
                error!("{e}, sequence: {}", header_meta.sequence());
            }
        }

        Ok(Empty)
    }
}
