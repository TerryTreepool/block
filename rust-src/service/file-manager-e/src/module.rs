
use std::sync::{Arc, RwLock, };

// use common::ModuleTrait;
use near_base::*;
use near_util::TopicRef;
use near_transport::{Stack, ProcessTrait, RoutineEventTrait, };

use base::ModuleTrait;
use nds::{NdsConfig, NdsStack, };

const SERVICE_NAME: &'static str = "file-manager";
// use crate::{files::Manager as FileManager, tasks::{Manager as TaskManager, ChunkReaderTrait, ChunkWriterTrait}};

struct ModuleImpl {
    #[allow(unused)]
    nds_stack: NdsStack,
}

#[derive(Clone)]
pub struct Module(Arc<ModuleImpl>);

impl Module {
    pub(super) fn new(stack: Stack) -> Self {
        let nds_stack = match NdsStack::open(SERVICE_NAME.to_owned(), stack, NdsConfig::default()) {
            Ok(stack) => stack,
            Err(err) => {
                panic!("{}", err);
            }
        };

        nds_stack.register_topic();

        Module(Arc::new(ModuleImpl{
            nds_stack,
        }))
    }

    pub fn nds_stack(&self) -> NdsStack {
        self.0.nds_stack.clone()
    }
}

lazy_static::lazy_static! {
    pub static ref RUNTIME_MODULE: RwLock<Option<Module>> = RwLock::new(None);
}

impl ModuleTrait for Module {
    fn clone_as_module(&self) -> Box<dyn ModuleTrait> {
        Box::new(self.clone())
    }
}

impl ProcessTrait for Module {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        self.nds_stack().clone_as_process()
    }

    fn create_routine(&self, from: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        self.nds_stack().create_routine(from, topic)
    }

}
