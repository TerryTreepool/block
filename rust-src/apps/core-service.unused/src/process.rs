
use std::{sync::Arc, path::PathBuf};

use near_base::{NearResult, ObjectId};
use near_core::get_service_path;
use near_util::TopicRef;
use near_transport::{ProcessTrait, RoutineEventTrait, };

use common::RuntimeProcessTrait;

use crate::event::{Manager as EventManager};

#[derive(Clone)]
struct Config {
    work_path: PathBuf,
}

struct ProcessComponents {
    event_manager: EventManager,
}

struct ProcessImpl {
    service_name: String,
    config: Config,

    components: Option<ProcessComponents>,
}

#[derive(Clone)]
pub struct Process(Arc<ProcessImpl>);

impl Process {
    pub fn new(service_name: &str) -> NearResult<Box<Self>> {
        let config = Config {
            work_path: get_service_path(service_name),
        };

        let ret = Self(Arc::new(ProcessImpl{
            service_name: service_name.to_owned(),
            config: config.clone(),
            components: None,
        }));

        let mut_ret = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut ProcessImpl) };
        mut_ret.components = Some(ProcessComponents {
            event_manager: EventManager::new(ret.clone()),
        });

        Ok(Box::new(ret))
    }

    #[inline]
    pub(crate) fn event_manager(&self) -> &EventManager {
        self.0.components.as_ref().unwrap()
    }
}

#[async_trait::async_trait]
impl RuntimeProcessTrait for Process {
    async fn run(&self) -> NearResult<()> {
        Ok(())
    }
}


impl ProcessTrait for Process {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, from: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        unimplemented!()
        // self.0.service.event_manager().create_routine(from, topic)
    }
}
