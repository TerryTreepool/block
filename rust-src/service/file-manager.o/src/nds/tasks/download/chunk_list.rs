
use std::sync::Arc;

use near_base::ChunkId;

use super::{super::{super::{inc::ChunkWriterTrait,
        },
        MultiDownloadSource,
        manager::TaskTrait,
    }};

struct ChunkListTaskImpl {
    chunks: Vec<ChunkId>,
    writer: Box<dyn ChunkWriterTrait>,
    source: MultiDownloadSource,
}

#[derive(Clone)]
pub struct ChunkListTask(Arc<ChunkListTaskImpl>);

impl ChunkListTask {
    pub fn new(chunks: Vec<ChunkId>, writer: Box<dyn ChunkWriterTrait>, source: MultiDownloadSource) -> Self {
        Self(Arc::new(ChunkListTaskImpl{
            chunks,
            writer,
            source,
        }))
    }
}

#[async_trait::async_trait]
impl TaskTrait for ChunkListTask {
    fn clone_as_task(&self) -> Box<dyn TaskTrait> {
        Box::new(self.clone())
    }

    async fn start(&self) {

    }
}
