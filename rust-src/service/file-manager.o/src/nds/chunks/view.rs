
use std::{sync::{RwLock, Arc}, };

use near_base::{ChunkId, NearResult, ErrorCode, NearError};

use crate::nds::nds_protocol::ChunkPieceDesc;

use super::{ChunkState, 
            super::inc::{ChunkReaderTrait, ChunkWriterTrait
            }, store::Manager,
        };

struct ChunkStateImpl {
    state: ChunkState,
    reader: Option<Box<dyn ChunkReaderTrait>>,
    writer: Option<Box<dyn ChunkWriterTrait>>,
    // upload_tasks: BTreeMap<ObjectId, ChunkUploader>,
    // writer: 
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
                reader: None,
                writer: None,
            })
        }))
    }

    pub fn chunk(&self) -> &ChunkId {
        &self.0.chunk
    }

    pub fn to_state(&self) -> ChunkState {
        self.0.state.read().unwrap().state
    }

    /// init chunk context, 
    /// create it's reader interface.
    pub(super) async fn load(&self) -> NearResult<()> {
        let (r, w) = Manager::get_instance().chunk_of(self.chunk()).await?;

        {
            let state = &mut *self.0.state.write().unwrap();

            state.reader = Some(r);
            state.writer = Some(w);
        }

        Ok(())
    }

    pub async fn read_to_end(&self, buf: &mut Vec<u8>, offset: usize, length: usize) -> NearResult<usize> {
        let r = self.0.state.read().unwrap().reader
                                               .as_ref()
                                               .map(|reader| {
                                                    reader.clone_as_reader()
                                                })
                                               .ok_or_else(|| {
                                                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Cloudn't found the [{}] chunk's reader", self.chunk()))
                                                })?;

        r.get(self.chunk(), offset, length)
         .await
         .map(| r | {
            buf.copy_from_slice(r);
            Ok(r.len())
         })?
    }

    pub async fn get_piece(&self, buf: &mut [u8], desc: &ChunkPieceDesc) -> NearResult<usize> {
        match desc {
            ChunkPieceDesc::Range(offset, count) => {
                {
                    self.0.state.read().unwrap()
                        .reader
                        .as_ref()
                        .map(| reader | reader.clone_as_reader() )
                        .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Cloudn't found the [{}] chunk's reader", self.chunk())))
                }?
                .get(self.chunk(), *offset as usize, *count as usize)
                .await
                .map(| r | {
                    buf[..].copy_from_slice(r);
                    Ok(r.len())
                })?
            }
        }
    }

    // pub fn start_upload(&self, task_id: SequenceValue, encode_codec: ChunkEncodeDesc, target: ObjectId, ) -> NearResult<Box<dyn TaskTrait>> {
    //     let chunk_uploader = {
    //         let state = &mut *self.0.state.write().unwrap();

    //         match &state.state {
    //             ChunkState::NotFound => Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("[{}] wasnot found.", self.chunk()))),
    //             ChunkState::Ready => {
    //                 let r = match state.upload_tasks.entry(target.clone()) {
    //                     Entry::Occupied(exist_node) => exist_node.get().clone(),
    //                     Entry::Vacant(new_node) => {
    //                         let uploader = ChunkUploader::new(self.clone());
    //                         new_node.insert(uploader.clone());
    //                         uploader
    //                     }
    //                 };
    //                 Ok(r)
    //             }
    //             _ => unreachable!(),
    //         }
    //     }?;

    //     chunk_uploader.start_upload(task_id, encode_codec, target)
    // }

}
