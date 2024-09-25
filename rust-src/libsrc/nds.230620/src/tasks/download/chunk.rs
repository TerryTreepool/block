
use std::sync::{Arc, };

use log::{info, error, };
use near_base::{ChunkId, ObjectId, NearResult, };

use crate::{nds_protocol::{PieceMessage, InterestMessage, SessionData, ChunkEncodeDesc}, 
            tasks::{ToSourceTrait, SessionTrait, manager::TaskTrait, }, 
            chunks::ChunkView, 
            inc::{ChunkWriterFeedbackTrait, ChunkWriterTrait, SaveToPathTrait}
        };

use super::{manager::Manager as DownloadManager, 
            DownloadRequestTrait, h::{DownloadTaskTrait, OnEventTrait}, encoder::{ChunkPieceRangeEncoder, FeedbackState}
        };

enum ChunkStateImpl {
    Pending(Arc<PendingState>),
}

struct PendingState {
    view: ChunkView,
    encoder: ChunkPieceRangeEncoder,
}

struct ChunkEvents {
    request: Box<dyn DownloadRequestTrait>,
    session: Option<Box<dyn SessionTrait>>,
    feedback: Box<dyn ChunkWriterFeedbackTrait>,
}

struct ChunkTaskImpl {
    manager: DownloadManager,
    task_id: u32,
    chunk: ChunkId,
    state: Option<ChunkStateImpl>,
    events: Option<ChunkEvents>,
}

#[derive(Clone)]
struct ChunkSessionDefault {
    session_id: u32,
    chunk_id: ChunkId,
}

impl<'a> ChunkSessionDefault {
    fn new(session_id: u32, chunk_id: ChunkId) -> Self {
        Self {
            session_id,
            chunk_id,
        }
    }
}

impl SessionTrait for ChunkSessionDefault {
    fn clone_as_session(&self) -> Box<dyn SessionTrait> {
        Box::new(self.clone())
    }

    fn object_id(&self) -> ObjectId {
        self.chunk_id.to_objectid()
    }

    fn session_id(&self) -> u32 {
        self.session_id
    }
}

#[derive(Clone)]
pub struct ChunkTask(Arc<ChunkTaskImpl>);

impl ChunkTask {
    pub fn new(manager: DownloadManager,
               chunk: ChunkId, 
               request: Box<dyn DownloadRequestTrait>,
               feedback: Box<dyn ChunkWriterFeedbackTrait>,
               session: Option<Box<dyn SessionTrait>>) -> NearResult<ChunkTask> {

        let task_id = manager.task_gen_id().generate().into_value();
        let ret = 
            Self(Arc::new(ChunkTaskImpl{
                manager,
                task_id,
                chunk,
                state: None,
                events: None,
            }));

        let ret_clone = ret.clone();
        let view = async_std::task::block_on(async move {
            match ret_clone.0.manager.nds_stack()
                    .chunk_manager()
                    .create_view(ret_clone.chunk(), crate::chunks::ChunkAccess::Write)
                    .await {
                Ok(view) => Ok(view),
                Err(err) => {
                    error!("failed [{}] create-view with err = {}", ret_clone.chunk(), err);
                    Err(err)
                }
            }
        })?;


        let events = ChunkEvents {
            request,
            session,
            feedback,
        };

        unsafe {
            let mut_self = &mut *(Arc::as_ptr(&ret.0) as *mut ChunkTaskImpl);
            mut_self.state = Some(ChunkStateImpl::Pending(Arc::new(PendingState { view: view.clone(), encoder: ChunkPieceRangeEncoder::new(ret.chunk(), view) })));
            mut_self.events = Some(events);
        };

        Ok(ret)
    }

    pub fn chunk(&self) -> &ChunkId {
        &self.0.chunk
    }

    #[inline]
    pub(self) fn event_request(&self) -> &dyn DownloadRequestTrait {
        self.0.events.as_ref().unwrap().request.as_ref()
    }

    #[inline]
    pub(self) fn event_session(&self) -> Box<dyn SessionTrait> {
        if let Some(session) = self.0.events.as_ref().unwrap().session.as_ref() {
            session.clone_as_session()
        } else {
            Box::new(ChunkSessionDefault::new(self.session_id(), self.chunk().clone()))
        }
    }

    #[inline]
    pub(self) fn event_feedback(&self) -> &dyn ChunkWriterFeedbackTrait {
        self.0.events.as_ref().unwrap().feedback.as_ref()
    }

    fn sync_state(&self) {
        let arc_self = self.clone();

        async_std::task::spawn(async move {
            let pending_state = arc_self.0.state.as_ref().unwrap();

            let state = match pending_state {
                ChunkStateImpl::Pending(state) => {
                    state.clone()
                }
            };

            match state.encoder.wait_finished().await {
                FeedbackState::Pending => {
                    unreachable!("impossible process")
                },
                FeedbackState::Finished => {
                    match arc_self.0.state.as_ref().unwrap() {
                        ChunkStateImpl::Pending(state) => {
                            let _ = state.view.save_to_path(&arc_self.0.manager.nds_stack().nds_config().data_path).await;
                            arc_self.event_feedback().finished(state.view.clone_as_writer()).await;
                        }
                    }
                },
                FeedbackState::Error(e) => {
                    arc_self.event_feedback().err(e).await;
                },
            }
        });
    }

}

impl std::fmt::Display for ChunkTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChunkTask::{{chunk:{}}}", self.0.chunk)
    }
}

impl DownloadTaskTrait for ChunkTask {
    fn clone_as_downloadtask(&self) -> Box<dyn DownloadTaskTrait> {
        Box::new(self.clone())
    }
}

impl SessionTrait for ChunkTask {
    fn clone_as_session(&self) -> Box<dyn SessionTrait> {
        Box::new(self.clone())
    }

    fn session_id(&self) -> u32 {
        self.0.task_id
    }

    fn object_id(&self) -> ObjectId {
        self.chunk().to_objectid()
    }
}

#[async_trait::async_trait]
impl TaskTrait for ChunkTask {

    async fn start(&self, source: Option<Box<dyn ToSourceTrait>>) {
        info!("{} with begin...", self );

        let source = match source {
            Some(source) => { source }
            None => {
                unreachable!("The chunk must srouce. fatal error.");
            }
        };

        let message = InterestMessage {
            session_data: SessionData {
                session_id: self.event_session().session_id(),
                session_sub_id: self.session_id(),    
            },
            chunk: self.chunk().clone(),
            encoder: ChunkEncodeDesc::create_stream(self.chunk()),
        };

        // sync state
        self.sync_state();

        // self.event_request().interest_chunk(source.source_of(0), self.chunk(), Some(self.event_session())).await;
        if let Err(err) = 
            self.event_request()
                .interest_chunk_v2(source.source_of(0), 
                                   Some(self.event_session().object_id()), 
                                   message)
                .await {
            error!("failed interest chunk object={} chunk={}, err={}", self.event_session().object_id(), self.chunk(), err);
        }

    }

}

#[async_trait::async_trait]
impl OnEventTrait for ChunkTask {
    async fn on_piece_data(&self, data: &PieceMessage) -> NearResult<()> {
        let pending_state = self.0.state.as_ref().unwrap();

        match pending_state {
            ChunkStateImpl::Pending(state) => {
                state.encoder.on_piece_data(data).await
            }
        }
    }
}
