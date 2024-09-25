use std::{sync::{Arc, RwLock, Mutex}, collections::LinkedList};

use near_base::{ObjectId, SequenceValue, NearResult};

use super::super::{manager::ChunkView,
        ChunkEncodeDesc,
    };
use super::{TaskTrait, AsyncChunkCursor, ChunkContent};


struct UploadTaskImpl {
    task_id: SequenceValue,
    #[allow(unused)]
    encode_codec: ChunkEncodeDesc,
    target: ObjectId,
    reader: Mutex<Box<dyn AsyncChunkCursor + Unpin + Send + Sync>>,
}

#[derive(Clone)]
pub struct UploadTask(Arc<UploadTaskImpl>);

impl UploadTask {
    pub fn new(task_id: SequenceValue, 
               encode_codec: ChunkEncodeDesc,
               target: ObjectId, 
               reader: ChunkContent) -> Self {
        Self(Arc::new(UploadTaskImpl{
            task_id,
            encode_codec,
            target,
            reader: Mutex::new(reader.into_cursor()),
        }))
    }
}

#[async_trait::async_trait]
impl TaskTrait for UploadTask {
    fn close_as_task(&self) -> Box<dyn TaskTrait> {
        Box::new(self.clone())
    }
}

struct ChunkUploaderImpl {
    view: ChunkView,
    tasks: RwLock<LinkedList<Box<dyn TaskTrait>>>,
}

#[derive(Clone)]
pub struct ChunkUploader(Arc<ChunkUploaderImpl>);

impl ChunkUploader {
    pub fn new(view: ChunkView) -> Self {
        Self(Arc::new(ChunkUploaderImpl{
            view,
            tasks: RwLock::new(LinkedList::new()),
        }))
    }

    pub fn start_upload(&self, task_id: SequenceValue, encode_codec: ChunkEncodeDesc, target: ObjectId) -> NearResult<Box<dyn TaskTrait>> {
        unimplemented!()
    }
}
