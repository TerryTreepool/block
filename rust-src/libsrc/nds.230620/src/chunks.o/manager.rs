
use std::{sync::{RwLock, Arc}, 
          collections::{BTreeMap, 
                        btree_map::Entry
    }};

use near_base::{file::FileObject, ChunkId, SequenceValue, ObjectId, NearResult, NearError, ErrorCode};

use crate::nds::NdsStack;

use super::{super::inc::{ChunkReaderTrait, ChunkWriterTrait}, ChunkState, ChunkEncodeDesc};
use super::{tasks::{TaskTrait, ChunkUploader}, };

struct FileTaskImpl {
    file: FileObject,
}

struct ChunkTaskImpl {
    chunk: ChunkId,
}

struct ChunkListTaskImpl {
    chunks: Vec<ChunkId>,
}

struct ChunkStateImpl {
    state: ChunkState,
    upload_tasks: BTreeMap<ObjectId, ChunkUploader>,
    download_tasks: BTreeMap<ObjectId, Box<dyn TaskTrait>>,
}

struct ChunkViewImpl {
    chunk: ChunkId,
    state: RwLock<ChunkStateImpl>,
}

#[derive(Clone)]
pub struct ChunkView(Arc<ChunkViewImpl>);

impl ChunkView {
    pub fn new(chunk: ChunkId) -> Self {
        Self(Arc::new(ChunkViewImpl{
            chunk,
            state: RwLock::new(ChunkStateImpl{
                        state: ChunkState::Unknown, 
                        upload_tasks: BTreeMap::new(),
                        download_tasks: BTreeMap::new(),
            })
        }))
    }

    pub fn chunk(&self) -> &ChunkId {
        &self.0.chunk
    }

    pub fn to_state(&self) -> ChunkState {
        self.0.state.read().unwrap().state
    }

    pub fn start_upload(&self, task_id: SequenceValue, encode_codec: ChunkEncodeDesc, target: ObjectId, ) -> NearResult<Box<dyn TaskTrait>> {
        let chunk_uploader = {
            let state = &mut *self.0.state.write().unwrap();

            match &state.state {
                ChunkState::NotFound => Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("[{}] wasnot found.", self.chunk()))),
                ChunkState::Ready => {
                    let r = match state.upload_tasks.entry(target.clone()) {
                        Entry::Occupied(exist_node) => exist_node.get().clone(),
                        Entry::Vacant(new_node) => {
                            let uploader = ChunkUploader::new(self.clone());
                            new_node.insert(uploader.clone());
                            uploader
                        }
                    };
                    Ok(r)
                }
                _ => unreachable!(),
            }
        }?;

        chunk_uploader.start_upload(task_id, encode_codec, target)
    }

}

pub struct Manager {
    stack: NdsStack,
    views: RwLock<BTreeMap<ChunkId, ChunkView>>,
    reader: Box<dyn ChunkReaderTrait>,
    writer: Box<dyn ChunkWriterTrait>,
}

impl Manager {
    pub fn new(stack: NdsStack, reader: Box<dyn ChunkReaderTrait>, writer: Box<dyn ChunkWriterTrait>) -> Self {
        Self {
            stack,
            views: RwLock::new(BTreeMap::new()),
            reader,
            writer,
        }
    }

    pub fn view_of(&self, chunk: &ChunkId) -> Option<ChunkView> {
        self.views.read().unwrap()
            .get(chunk)
            .map(| view | view.clone())
    }

    pub fn create_view(&self, chunk: ChunkId) -> ChunkView {
        let views = &mut *self.views.write().unwrap();

        let r = match views.entry(chunk.clone()) {
            Entry::Occupied(exist) => {
                exist.get().clone()
            }
            Entry::Vacant(a) => {
                let view = ChunkView::new(chunk);
                a.insert(view.clone());
                view
            }
        };

        r
    }

    pub(super) fn reader(&self) -> &dyn ChunkReaderTrait {
        self.reader.as_ref()
    }

    pub(super) fn writer(&self) -> &dyn ChunkWriterTrait {
        self.writer.as_ref()
    }

}

impl Manager {
    pub async fn start_upload(&self, task_id: SequenceValue, chunk: ChunkId, encode_codec: ChunkEncodeDesc, target: ObjectId, ) -> NearResult<()> {
        let task = 
            self.create_view(chunk)
                .start_upload(task_id, encode_codec, target)
                .map(| task | {
                    task.close_as_task()
                })
                .map_err(| err | {
                    // error!("failed start_upload() with err {}", err);
                    println!("failed start_upload() with err {}", err);
                    err
                })?;

        // start upload task

        Ok(())
    }

}
