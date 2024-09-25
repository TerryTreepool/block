
use std::{sync::Arc, path::PathBuf, };

use base::{MessageExpire, SubscribeMessage, DissubcribeMessage, TopicRouineManager, TopicRoutineOpEventTrait, };
use log::{trace};
use near_base::{NearResult, ObjectId, NearError, ErrorCode, file::FileObject, ChunkId, };
use near_core::get_data_path;
use near_transport::{Stack as BaseStack, ProcessTrait, RoutineEventTrait, };
use near_util::{FileBuilder, TOPIC_CORE_SUBSCRIBE, TOPIC_CORE_DISSUBSCRIBE};
use near_util::{Topic, TopicBuilder, TopicStruct, TopicRef, };

use crate::{tasks::DownloadRequestTrait, 
            inc::{PRIMARY_TOPIC_NDS_LABEL, SECONDARY_TOPIC_NDS_FILE_LABEL, SECONDARY_TOPIC_NDS_INTEREST, SECONDARY_TOPIC_NDS_PIECE},
            tasks::{Manager as TaskManager, DownloadSourceRef, SessionTrait},
            chunks::{Manager as ChunkManager},
            nds_protocol::{PieceMessageBuilder, SyncFileMessage, InterestMessage, SessionData, ChunkEncodeDesc, },
            stack_private::{OnNdsSyncFile, OnNdsInterest, OnNdsPieceData},
    };

lazy_static::lazy_static! {
    static ref TOPIC_NDS_SYNC_FILE_PRI: Topic = TopicBuilder::new(PRIMARY_TOPIC_NDS_LABEL).secondary(SECONDARY_TOPIC_NDS_FILE_LABEL).build();
    pub static ref TOPIC_NDS_SYNC_FILE: TopicStruct<'static> = {
        let topic: &'static Topic = &TOPIC_NDS_SYNC_FILE_PRI;
        TopicStruct::try_from(topic).unwrap()
    };

    static ref TOPIC_NDS_INTEREST_CHUNK_PRI: Topic = TopicBuilder::new(PRIMARY_TOPIC_NDS_LABEL).secondary(SECONDARY_TOPIC_NDS_INTEREST).build();
    static ref TOPIC_NDS_INTEREST_CHUNK: TopicStruct<'static> = {
        let topic: &'static Topic = &TOPIC_NDS_INTEREST_CHUNK_PRI;
        TopicStruct::try_from(topic).unwrap()
    };

    static ref TOPIC_NDS_PIECE_DATA_PRI: Topic = TopicBuilder::new(PRIMARY_TOPIC_NDS_LABEL).secondary(SECONDARY_TOPIC_NDS_PIECE).build();
    pub static ref TOPIC_NDS_PIECE_DATA: TopicStruct<'static> = {
        let topic: &'static Topic = &TOPIC_NDS_PIECE_DATA_PRI;
        TopicStruct::try_from(topic).unwrap()
    };
}

#[derive(Clone)]
pub struct Config {
    #[allow(unused)]
    /// chunk cache save path
    pub data_path: PathBuf,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self{
            data_path: get_data_path(),
        }
    }
}

struct StackComponents {
    // file_manager: FileManager,
    task_manager: TaskManager,
    chunk_manager: ChunkManager,
    topic_manager: TopicRouineManager,
}

struct StackImpl {
    service_name: String,
    stack: BaseStack,
    #[allow(unused)]
    config: Config,
    components: Option<StackComponents>,
}

#[derive(Clone)]
pub struct Stack(Arc<StackImpl>);

impl Stack {
    pub fn open(name: String, stack: BaseStack, config: Config) -> NearResult<Self> {
        let ret = Self(Arc::new(StackImpl{
            service_name: name,
            stack,
            config: config.clone(),
            components: None,
        }));

        let chunk_manager = ChunkManager::new(ret.clone());

        let task_manager = TaskManager::new(ret.clone(), None)?;

        let topic_manager = TopicRouineManager::new(Box::pin(ret.clone()));

        let c = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut StackImpl) };
        c.components = Some(StackComponents {
            task_manager,
            chunk_manager,
            topic_manager,
        });

        Ok(ret)
    }

    #[inline]
    pub(crate) fn service_name(&self) -> &str {
        self.0.service_name.as_str()
    }

    #[inline]
    pub(crate) fn task_manager(&self) -> &TaskManager {
        &self.0.components.as_ref().unwrap().task_manager
    }

    #[inline]
    pub(crate) fn chunk_manager(&self) -> &ChunkManager {
        &self.0.components.as_ref().unwrap().chunk_manager
    }

    #[inline]
    pub(crate) fn topic_manager(&self) -> &TopicRouineManager {
        &self.0.components.as_ref().unwrap().topic_manager
    }

    #[inline]
    pub(crate) fn base_stack(&self) -> &BaseStack {
        &self.0.stack
    }

    #[inline]
    pub(crate) fn nds_config(&self) -> &Config {
        &self.0.config
    }

}

impl Stack {

    pub fn register_topic(&self) {
        let arc_self = self.clone();
        async_std::task::spawn(async move {
            arc_self.topic_manager()
                    .register_topic_event(TOPIC_NDS_SYNC_FILE.topic(), 
                                          OnNdsSyncFile::new(arc_self.clone()))
                    .await;

            arc_self.topic_manager()
                    .register_topic_event(TOPIC_NDS_PIECE_DATA.topic(), 
                                          OnNdsPieceData::new(arc_self.clone()))
                    .await;
        });
    }

    pub async fn track_from_file(&self, path: &PathBuf) -> NearResult<()> {
        if !path.exists() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("[{}] is not exists.", path.to_str().unwrap_or("None"))));
        }

        let file = FileBuilder::new(path)
                    .author(Some(self.0.stack.local_device_id()))
                    .build()
                    .await
                    .map_err(| err | {
                        // near_error!(err)
                        err
                    })?;

        self.chunk_manager().track_file(&file, path).await?;

        self.sync_file(file).await?;

        Ok(())
    }

    pub(self) async fn sync_file(&self, file: FileObject) -> NearResult<()> {
        let topic = TopicBuilder::from(TOPIC_NDS_INTEREST_CHUNK.topic_ref())
                                .add_thirdary(file.object_id().to_string().as_str())
                                .build();
        self.topic_manager().register_topic_event(&topic, OnNdsInterest::new(self.clone())).await;

        self.base_stack()
            .post_message(None, 
                          TOPIC_NDS_SYNC_FILE.topic().clone(), 
                          SyncFileMessage {
                            file
                          },
                          None)
    }
}

impl TopicRoutineOpEventTrait for Stack {
    fn subscribe_message(&self, topic: &Topic) -> NearResult<()> {
        self.base_stack()
            .post_message(None, 
                          TOPIC_CORE_SUBSCRIBE.topic().clone(), 
                          SubscribeMessage {
                            message_list: vec![(topic.topic().clone().into(), MessageExpire::Forever)],
                          },
                          None)
    }

    fn dissubscribe_message(&self, topic: &Topic) -> NearResult<()> {
        self.base_stack()
            .post_message(None, 
                          TOPIC_CORE_DISSUBSCRIBE.topic().clone(), 
                          DissubcribeMessage {
                            message: topic.topic().clone().into(),
                          },
                          None)
    }
}

impl ProcessTrait for Stack {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, from: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        trace!("from: {}, topic: {}, ", from, topic);

        self.topic_manager().call(&topic)
    }

}

// OnDownloadEventTrait
#[async_trait::async_trait]
impl DownloadRequestTrait for Stack {

    async fn interest_chunk(&self, target: DownloadSourceRef, chunk: &ChunkId, session: Option<Box<dyn SessionTrait>>) {
        // let session = match session {
        //     Some(session) => session,
        //     None => {
        //         error!("Missing session, session is None.");
        //         return;
        //     }
        // };

        // let new_topic = TopicBuilder::from(TOPIC_NDS_INTEREST_CHUNK.topic_ref())
        //                             .add_thirdary(session.object_id().to_string().as_str())
        //                             .build();

        // if let Err(err) = 
        //     self.base_stack()
        //         .post_message(
        //             None, 
        //             new_topic, 
        //             InterestMessage {
        //                 session_id: session.session_id(),
        //                 chunk: chunk.clone(),
        //                 desc: PieceEncodeDesc::Range(0, chunk.len() as u32),
        //             },
        //             None) {
        //         error!("failed post-message topic={}, chunk={}, err={}", TOPIC_NDS_INTEREST_CHUNK.topic_ref(), chunk, err);
        //     }
    }

    async fn interest_chunk_v2(&self, target: DownloadSourceRef, object_id: Option<ObjectId>, message: InterestMessage) -> NearResult<()> {

        let new_topic = if let Some(object_id) = object_id {
            TopicBuilder::from(TOPIC_NDS_INTEREST_CHUNK.topic_ref())
                .add_thirdary(object_id.to_string().as_str())
                .build()
        } else {
            TOPIC_NDS_INTEREST_CHUNK_PRI.clone()
        };

        self.base_stack()
            .post_message(None, 
                          new_topic, 
                          message,
                          None)
    }

}

// UploadEventTrait
impl Stack {
    pub fn push_piece_data(&self, 
                           target: &ObjectId,
                           session_data: SessionData,
                           chunk: &ChunkId, 
                           encoder: ChunkEncodeDesc, 
                           data: Vec<u8>) -> NearResult<()> {
        let b = PieceMessageBuilder {
            session_data,
            chunk,
            encoder,
            data,
        };

        self.0
            .stack
            .post_message_with_builder(Some(target.clone()), 
                                       TOPIC_NDS_PIECE_DATA.topic(),
                                       b, 
                                       None)
    }
}
 