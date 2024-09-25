
use std::{sync::Arc, str::FromStr, collections::HashMap};

use log::{trace, error};

use near_base::NearResult;
use near_transport::{Routine, RoutineEventTrait, RoutineWrap, HeaderMeta, EventResult};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, try_decode_raw_object};
use topic_util::types::{hci_types::HciTaskId, thing_data::ThingId};

use crate::{tasks::{TaskData, TaskModule}, process::Process};

struct CrudThingTaskRoutineImpl {
    process: Process,
    task_module: TaskModule,
}

#[derive(Clone)]
pub struct CrudThingTaskRoutine(Arc<CrudThingTaskRoutineImpl>);

impl CrudThingTaskRoutine {
    pub fn new(process: Process, task_module: TaskModule) -> Box<dyn RoutineEventTrait> {
        debug_assert_eq!(task_module.into_value(), TaskModule::Search.into_value());

        let ret = Self(Arc::new(CrudThingTaskRoutineImpl{ 
            process,
            task_module,
        }));

        RoutineWrap::new(Box::new(ret))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for CrudThingTaskRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("CrudThingTaskRoutine::on_routine task_module={}, header_meta={header_meta}", self.0.task_module.to_str());

        let r = 
            try_decode_raw_object!((String, HashMap<String, String>), req, o, o, { header_meta.sequence() });

        let r: DataContent<HciTaskId> = match r {
            DataContent::Content((thing_id, thing_data)) => {
                self.add_thing_task(header_meta, thing_id, thing_data).await
            }
            DataContent::Error(e) => Err(e)
        }.into();

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl CrudThingTaskRoutine {
    pub(in self) async fn add_thing_task(
        &self, 
        header_meta: &HeaderMeta, 
        thing_id: String, 
        thing_data: HashMap<String, String>
    ) -> NearResult<HciTaskId> {
        let _ = 
            ThingId::from_str(&thing_id)
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        let thing = 
            self.0.process
                .thing_components()
                .get_thing_by_id(&thing_id)
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        let mut thing = thing.thing().clone();

        thing.mut_body().mut_content().mut_user_data().extend(thing_data);

        let task_id = 
            self.0.process.task_manager().add_task(
                TaskData::from((
                    self.0.task_module.clone(),
                    &thing
                ))
            ).await?;

        match self.0.task_module {
            TaskModule::AddThing => self.0.process.thing_components().add_things(vec![thing].into_iter()),
            TaskModule::RemoveThing => self.0.process.thing_components().remove_thing(thing.object_id()),
            _ => { /* ignore */ }
        }

        Ok(task_id)
    }
}
