
use std::sync::{Arc, RwLock};

use log::{info, error};
use near_base::{ChunkId, ObjectId, NearResult};

use crate::nds_protocol::PieceMessage;

use super::{super::{super::{chunks::ChunkView, 
                            inc::ChunkWriterTrait,
                    },
                    SingleDownloadSource,
                    manager::{TaskTrait, },
                }, 
        manager::Manager as DownloadManager, OnEventTrait,
    };

enum ChunkStateImpl {
    Prepair,
    Pending(Arc<PendingState>),
}

struct PendingState {
    view: ChunkView,
}

pub trait ChunkTaskWriterTrait: ChunkWriterTrait + Send + Sync {
    fn file_id(&self) -> &ObjectId;
}

struct ChunkTaskImpl {
    manager: DownloadManager,
    task_id: u32,
    chunk: ChunkId,
    // source: SingleDownloadSource,
    state: RwLock<ChunkStateImpl>,
    writer: Box<dyn ChunkTaskWriterTrait>,
}

#[derive(Clone)]
pub struct ChunkTask(Arc<ChunkTaskImpl>);

impl ChunkTask {
    pub fn new(manager: DownloadManager,
               chunk: ChunkId, 
            //    source: SingleDownloadSource,
               writer: Box<dyn ChunkTaskWriterTrait>) -> NearResult<ChunkTask> {

        let task_id = manager.task_gen_id().generate().into_value();

        Ok(Self(Arc::new(ChunkTaskImpl{
            manager,
            task_id,
            chunk,
            // source,
            state: RwLock::new(ChunkStateImpl::Prepair),
            writer,
        })))
    }

    pub fn chunk(&self) -> &ChunkId {
        &self.0.chunk
    }
}

impl std::fmt::Display for ChunkTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChunkTask::{{chunk:{}}}", self.0.chunk)
    }
}

#[async_trait::async_trait]
impl TaskTrait for ChunkTask {
    fn clone_as_task(&self) -> Box<dyn TaskTrait> {
        Box::new(self.clone())
    }

    fn task_id(&self) -> u32 {
        self.0.task_id
    }

    async fn start(&self) {
        info!("{} begin...", self);

        let view = match self.0.manager.nds_stack()
                                        .chunk_manager()
                                        .create_view(self.chunk(), crate::chunks::ChunkAccess::Write)
                                        .await {
            Ok(view) => view,
            Err(err) => {
                error!("failed [{}] create-view with err = {}", self.chunk(), err);
                let _ = self.0.writer.err(err.errno());
                return;
            }
        };

        let pending_state = {
            let state = &mut *self.0.state.write().unwrap();

            match state {
                ChunkStateImpl::Prepair => {
                    let pending_state = Arc::new(PendingState {
                        view,
                    });
                    *state = ChunkStateImpl::Pending(pending_state.clone());
                    pending_state
                }
                ChunkStateImpl::Pending(_) => {
                    info!("[{}] task has been running.", self.chunk());
                    return;
                }
            }
        };

        let source = self.0.source.as_ref().target().clone();
        self.0.manager.nds_stack().interest_chunk(source, self.0.writer.file_id(), self.chunk());
    }

}

#[async_trait::async_trait]
impl OnEventTrait for ChunkTask {
    async fn on_piece_data(&self, data: &PieceMessage) -> NearResult<()> {
        Ok(())
    }
}
