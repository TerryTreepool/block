
use std::{sync::Arc, path::PathBuf, };

use near_base::{NearResult, ObjectId, };
use near_transport::{Stack as NearStack, ProcessTrait, RoutineEventTrait, topic::{Topic, TopicRef}};

use crate::nds::{stack_private::OnNdsSyncFile, };

use super::{/* files::{Manager as FileManager}, */
            inc::{PRIMARY_TOPIC_NDS_LABEL, 
                  SECONDARY_TOPIC_NDS_FILE_LABEL, SECONDARY_TOPIC_NDS_INTEREST, SECONDARY_TOPIC_NDS_PIECE}, 
            tasks::{Manager as TaskManager, UploadEventTrait, },
            topic_routine, nds_protocol::{ChunkPieceDesc, PieceMessageBuilder},
    };

lazy_static::lazy_static! {
    static ref TOPIC_NDS_SYNC_FILE: Topic = Topic::from((PRIMARY_TOPIC_NDS_LABEL, SECONDARY_TOPIC_NDS_FILE_LABEL));
    static ref TOPIC_NDS_INTEREST_PIECE: Topic = Topic::from((PRIMARY_TOPIC_NDS_LABEL, SECONDARY_TOPIC_NDS_INTEREST));
    static ref TOPIC_NDS_PIECE_DATA: Topic = Topic::from((PRIMARY_TOPIC_NDS_LABEL, SECONDARY_TOPIC_NDS_PIECE));
}

#[derive(Clone)]
pub struct Config {
    #[allow(unused)]
    cache_path: Option<PathBuf>,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self{
            cache_path: None
        }
    }
}

struct StackComponents {
    // file_manager: FileManager,
    task_manager: TaskManager,
    // chunk_manager: ChunkManager,
    topic_manager: topic_routine::Manager,
}

struct StackImpl {
    stack: NearStack,
    #[allow(unused)]
    config: Config,
    components: Option<StackComponents>,
}

#[derive(Clone)]
pub struct Stack(Arc<StackImpl>);

impl Stack {
    pub fn open(stack: NearStack, config: Config) -> NearResult<Self> {
        let ret = Self(Arc::new(StackImpl{
            stack,
            config: config.clone(),
            components: None,
        }));

        // let file_manager = FileManager::new(ret.clone(), config.cache_path.as_ref().map(|path| path.clone()));
        // let chunk_manager = ChunkManager::new(ret.clone());

        let task_manager = TaskManager::new(ret.clone(), None)?;

        let topic_manager = ret.init_topic_manager();

        let c = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut StackImpl) };
        c.components = Some(StackComponents {
            // file_manager,
            task_manager,
            topic_manager,
        });

        Ok(ret)
    }

    pub(crate) fn task_manager(&self) -> &TaskManager {
        &self.0.components.as_ref().unwrap().task_manager
    }

    pub(crate) fn topic_manager(&self) -> &topic_routine::Manager {
        &self.0.components.as_ref().unwrap().topic_manager
    }

}

impl Stack {
    fn init_topic_manager(&self) -> topic_routine::Manager {
        let topic_manager = topic_routine::Manager::default();

        topic_manager.register_topic_event(TOPIC_NDS_SYNC_FILE.clone(), OnNdsSyncFile::new(self.clone()));
        topic_manager.register_topic_event(TOPIC_NDS_INTEREST_PIECE.clone(), OnNdsSyncFile::new(self.clone()));

        topic_manager
    }

}

impl ProcessTrait for Stack {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, from: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        println!("from: {}, topic: {}, ", from, topic);

        self.topic_manager().call(&topic)
    }

}

impl UploadEventTrait for Stack {
    fn push_piece_data(&self, 
                       target: ObjectId,
                       chunk: near_base::ChunkId, 
                       desc: ChunkPieceDesc, 
                       data: Vec<u8>) -> NearResult<()> {
        let b = PieceMessageBuilder {
            chunk,
            offset: if let Some((offset, _)) = desc.to_range() {
                offset as usize
            } else {
                0
            },
            data,
        };

        self.0.stack.post_message_with_builder(Some(target), 
                                               &TOPIC_NDS_PIECE_DATA, 
                                               b, 
                                               None)
    }
}
