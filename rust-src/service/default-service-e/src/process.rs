
use std::{sync::Arc, cell::UnsafeCell, path::PathBuf};

use common::RuntimeProcessTrait;
use log::trace;
use near_core::get_data_path;
use near_util::TopicRef;
use rlua::Lua;

use near_base::{ObjectId, NearResult};
use near_transport::{ProcessTrait, RoutineEventTrait};

struct Config {
    work_path: PathBuf,
}

struct ProcessImpl {
    service_name: String,
    lua: UnsafeCell<Lua>,
    config: Config,
}

#[derive(Clone)]
pub struct Process(Arc<ProcessImpl>);

impl Process {
    pub fn new(service_name: &str) -> Self {
        Self(Arc::new(ProcessImpl{
            service_name: service_name.to_owned(),
            lua: UnsafeCell::new(Lua::new()),
            config: Config {
                work_path: get_data_path().join(service_name),
            },
        }))
    }
}

unsafe impl Sync for Process {}
unsafe impl Send for Process {}

#[async_trait::async_trait]
impl RuntimeProcessTrait for Process {

    async fn run(&self) -> NearResult<()> {
        trace!("run...");

        Ok(())
    }
    
}

impl ProcessTrait for Process {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, sender: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        unimplemented!()
    }

}
