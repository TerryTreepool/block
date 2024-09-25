
use base::ModuleTrait;
use near_core::get_data_path;
use once_cell::sync::OnceCell;
use std::sync::{Arc, RwLock, };

// use common::ModuleTrait;
use near_base::*;
use near_transport::{Stack, ProcessTrait, RoutineEventTrait, topic::TopicRef, };

use crate::gateway::{App, Config, GATEWAY_DATA_DB};

struct ModuleImpl {
    #[allow(unused)]
    app: App,
}

#[derive(Clone)]
pub struct Module(Arc<ModuleImpl>);

impl Module {
    pub(crate) fn new(stack: Stack) -> Self {

        Self(Arc::new(ModuleImpl{
            app: App::new(stack, Config::default().db(get_data_path().join(GATEWAY_DATA_DB))).unwrap(),
        }))
    }
}

impl ModuleTrait for Module {
    fn clone_as_module(&self) -> Box<dyn ModuleTrait> {
        Box::new(self.clone())
    }
}

impl ProcessTrait for Module {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        self.0.app.nds_stack().clone_as_process()
    }

    fn create_routine(&self, from: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        self.0.app.nds_stack().create_routine(from, topic)
    }

}
