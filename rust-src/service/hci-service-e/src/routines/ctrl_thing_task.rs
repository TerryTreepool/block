
use std::{sync::Arc, collections::HashMap, time::Duration};

use log::{trace, error};

use near_base::{NearResult, NearError, ErrorCode};
use near_transport::{Routine, RoutineEventTrait, RoutineWrap, HeaderMeta, EventResult};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, try_decode_raw_object};
use topic_util::types::hci_types::HciTaskId;

use crate::{process::Process, tasks::{TaskModule, TaskData}, };

#[derive(Clone)]
pub struct Config {
    pub ctrl_interval: Duration,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            ctrl_interval: std::time::Duration::from_millis(3),
        }
    }
}

struct ControlThingTaskRoutineImpl {
    process: Process,
}

#[derive(Clone)]
pub struct ControlThingTaskRoutine(Arc<ControlThingTaskRoutineImpl>);

impl ControlThingTaskRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        let ret = Self(Arc::new(ControlThingTaskRoutineImpl{ 
            process,
        }));

        RoutineWrap::new(Box::new(ret))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for ControlThingTaskRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("ControlThingTaskRoutine::on_routine header_meta={header_meta}");

        let r = 
            try_decode_raw_object!(Vec<(String, HashMap<String, String>)>, req, o, o, { header_meta.sequence() });

        let r: DataContent<HciTaskId> = match r {
            DataContent::Content(things) => 
                self.on_routine(header_meta, things).await.into(),
            DataContent::Error(e) => 
                DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl ControlThingTaskRoutine {
    pub(in self) async fn on_thing_routine(&self, header_meta: &HeaderMeta, thing_id: String, thing_data: HashMap<String, String>) -> NearResult<()> {
        trace!("ControlThingTaskRoutine::on_thing_routine thing_id={thing_id}");

        if let Ok(thing) = self.0.process.thing_components().get_thing_by_id(&thing_id) {
            let mut thing = thing.thing().clone();

            thing.mut_body().mut_content().mut_user_data().extend(thing_data);

            self.0.process.task_manager()
                .add_task(
                    TaskData::from((
                        TaskModule::ControlThing,
                        thing
                    ))
                )
                .await
                .map(| _ | ())
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })

        } else {
            let error_string = format!("Not found [{thing_id}] thing.");
            error!("{error_string}, sequence: {}", header_meta.sequence());
            Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string))
        }

    }

    pub(in self) async fn on_routine(&self, header_meta: &HeaderMeta, things: Vec<(String, HashMap<String, String>)>) -> NearResult<HciTaskId> {

        let config = &self.0.process.config().routines_config.ctrl_config;

        if config.ctrl_interval > Duration::ZERO {
            for (thing_id, thing_data) in things {
                let _ = self.on_thing_routine(header_meta, thing_id, thing_data).await;
                let _ = async_std::future::timeout(config.ctrl_interval, async_std::future::pending::<()>()).await;
            }
        } else {
            for (thing_id, thing_data) in things {
                let _ = self.on_thing_routine(header_meta, thing_id, thing_data).await;
            }
        }

        Ok(TaskModule::QueryThing.into_value())
    }
}
