
use std::sync::Arc;

use log::{trace, error};

use near_base::{NearResult, builder_codec_macro::Empty, ObjectId, now};
use near_transport::{Routine, RoutineEventTrait, RoutineWrap, HeaderMeta, EventResult};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, try_decode_raw_object};

use crate::{process::Process, tasks::{TaskModule, TaskData}, cache::{thing_components::ThingComponentPtr, ThingStatus}, };

#[derive(Clone)]
pub struct QueryTaskConfig {
    pub(crate) timeout_response: std::time::Duration,

}

impl std::default::Default for QueryTaskConfig {
    fn default() -> Self {
        Self {
            timeout_response: std::time::Duration::from_secs(300),
        }
    }
}


struct QueryAllThingTaskRoutineImpl {
    process: Process,
}

#[derive(Clone)]
pub struct QueryAllThingTaskRoutine(Arc<QueryAllThingTaskRoutineImpl>);

impl QueryAllThingTaskRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        let ret = Self(Arc::new(QueryAllThingTaskRoutineImpl{ 
            process,
        }));

        RoutineWrap::new(Box::new(ret))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for QueryAllThingTaskRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("QueryAllThingTaskRoutine::on_routine header_meta={header_meta}");

        let r = try_decode_raw_object!(Empty, req, o, o, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content(_) => self.on_routine(header_meta).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryAllThingTaskRoutine {
    pub(in self) async fn on_thing_routine(&self, header_meta: &HeaderMeta, thing: ThingComponentPtr) -> NearResult<()> {
        trace!("QueryAllThingTaskRoutine::on_thing_routine thing_id={}", thing.thing().object_id());

        self.0.process.task_manager()
            .add_task(
                TaskData::from((
                    TaskModule::QueryThing,
                    thing.thing()
                ))
            )
            .await
            .map(| _ | ())
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })

    }

    pub(in self) async fn on_routine(&self, header_meta: &HeaderMeta) -> NearResult<Empty> {
        let timeout_response = self.0.process.config().routines_config.query_task_config.timeout_response.as_micros() as u64;
        let now = now();

        let things = self.0.process.thing_components().get_all_thing();
        let mut fut = vec![];

        // check status
        {
            let timeout_things: Vec<&ObjectId> = 
            things.iter()
                .filter(| it | {
                    match it.status() {
                        ThingStatus::Online(last_updated, _) => {
                            (now - last_updated) > timeout_response
                        }
                        _ => true
                    }
                })
                .map(| it | {
                    it.thing().object_id()
                })
                .collect();

            self.0.process.thing_components().offline(timeout_things.into_iter());
        }


        for thing in things {

            fut.push(
                self.on_thing_routine(header_meta, thing)
            );
        }

        let _ = futures::future::join_all(fut).await;

        Ok(Empty)
    }

}
