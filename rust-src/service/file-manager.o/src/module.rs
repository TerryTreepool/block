
use std::sync::{Arc, RwLock, };

use log::{info, error, };

use common::ModuleTrait;
use near_base::*;
use near_base::file::FileObject;
use near_transport::{Stack, ProcessTrait, RoutineEventTrait, Routine, EventResult, RoutineWrap, ResponseEvent, topic::TopicRef, };
use protocol::SubscribeMessage;

use crate::nds::{NdsConfig, NdsStack, MultiDownloadSource, SingleDownloadSource, DownloadSource};

// use crate::{files::Manager as FileManager, tasks::{Manager as TaskManager, ChunkReaderTrait, ChunkWriterTrait}};

struct ModuleImpl {
    #[allow(unused)]
    nds_stack: NdsStack,
    // file_manager: FileManager,
    // #[allow(unused)]
    // chunk_manager: ChunkManager,
}

#[derive(Clone)]
pub struct Module(Arc<ModuleImpl>);

impl Module {
    pub(super) fn new(stack: Stack) -> Self {
        // let stack_clone = stack.clone();
        // async_std::task::spawn(async move {

        //     match stack_clone.post_message(None,
        //                                    TOPIC_SUBSCRIBE.as_str(),
        //                                    SubscribeMessage {
        //                                         message_list: vec!["/upload/sync-file".to_string(),
        //                                                            "/upload/sync-chunk".to_string(),
        //                                                            "/upload/sync-chunklist".to_string()],
        //                                    },
        //                                    None) {
        //         Ok(_) => {
        //             info!("Succeed subscribe file-manager's topic");
        //             Ok(())
        //         }
        //         Err(err) => {
        //             error!("Failed subscribe topic with {}", err);
        //             Err(err)
        //         }
        //     }

        // });

        // let file_manager = FileManager::new(None);
        let nds_stack = match NdsStack::open(stack, NdsConfig::default()) {
            Ok(stack) => stack,
            Err(err) => {
                panic!("{}", err);
            }
        };

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
