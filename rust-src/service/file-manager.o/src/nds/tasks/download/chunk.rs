
use std::sync::Arc;

use near_base::ChunkId;

use super::{super::{super::{chunks::ChunkView, 
                    },
                    SingleDownloadSource,
                    manager::{TaskTrait, },
                }, 
    };

struct ChunkTaskImpl {
    chunk: ChunkId,
    source: SingleDownloadSource,
    view: ChunkView,
    // interest_event: Box<dyn OnEventTrait>,
}

#[derive(Clone)]
pub struct ChunkTask(Arc<ChunkTaskImpl>);

impl ChunkTask {
    // pub fn new(chunk: ChunkId, source: SingleDownloadSource, view: ChunkView, event: Box<dyn OnEventTrait>) -> Self {
    //     Self(Arc::new(ChunkTaskImpl{
    //         chunk,
    //         source,
    //         view,
    //         interest_event: event
    //     }))
    // }

    pub fn chunk(&self) -> &ChunkId {
        &self.0.chunk
    }
}

#[async_trait::async_trait]
impl TaskTrait for ChunkTask {
    fn clone_as_task(&self) -> Box<dyn TaskTrait> {
        Box::new(self.clone())
    }

    async fn start(&self) {
        // self.0.interest_event.interest(Interest{
        //                                     chunk: self.chunk().clone(), 
        // });
    }

}
