
use std::sync::Arc;

use log::{trace, error};

use near_base::{NearResult, thing::ThingObject};
use near_transport::{Routine, RoutineEventTrait, RoutineWrap, HeaderMeta, EventResult};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, try_decode_raw_object};
use topic_util::types::hci_types::HciTaskId;

use crate::{tasks::{TaskData, TaskModule}, process::Process};

struct AddThingTaskRoutineImpl {
    process: Process,
}

#[derive(Clone)]
pub struct AddThingTaskRoutine(Arc<AddThingTaskRoutineImpl>);

impl AddThingTaskRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        let ret = Self(Arc::new(AddThingTaskRoutineImpl{ 
            process,
        }));

        RoutineWrap::new(Box::new(ret))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for AddThingTaskRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("AddThingTaskRoutine::on_routine header_meta={header_meta}");

        let r = 
            try_decode_raw_object!(ThingObject, req, o, o, { header_meta.sequence() });

        let r: DataContent<HciTaskId> = match r {
            DataContent::Content(thing_id) => {
                self.add_thing_task(header_meta, thing_id).await
            }
            DataContent::Error(e) => Err(e)
        }.into();

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl AddThingTaskRoutine {
    pub(in self) async fn add_thing_task(&self, header_meta: &HeaderMeta, thing_object: ThingObject) -> NearResult<HciTaskId> {

        let task_id = 
            self.0.process.task_manager().add_task(
                TaskData::from((
                    TaskModule::AddThing.into(),
                    &thing_object
                ))
            )
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        self.0.process.thing_components().add_things(vec![thing_object].into_iter());

        Ok(task_id)
    }
}
