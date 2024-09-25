
use std::sync::{Arc, };

use log::error;
use near_base::{ObjectId, NearResult, };

use crate::{tasks::{ToSourceTrait, 
                    SessionTrait, 
                    manager::TaskTrait, 
                    upload::UploadManager,
                }, 
            chunks::ChunkView, 
            nds_protocol::{SessionData, ChunkEncodeDesc, PieceMessage}, 
    };

use super::UploadTaskTrait;

struct ChunkTaskImpl {
    manager: UploadManager,
    session_data: SessionData,
    view: ChunkView,
    encoder: ChunkEncodeDesc,
    target: ObjectId,
}

#[derive(Clone)]
pub struct ChunkTask(Arc<ChunkTaskImpl>);

impl ChunkTask {
    pub fn new(manager: UploadManager, session_data: SessionData, view: ChunkView, target: ObjectId, encoder: ChunkEncodeDesc) -> NearResult<Self> {
        Ok(Self(Arc::new(ChunkTaskImpl{
            manager,
            session_data,
            view, 
            encoder,
            target,
        })))
    }

}

impl UploadTaskTrait for ChunkTask {
    fn clone_as_uploadtask(&self) -> Box<dyn UploadTaskTrait> {
        Box::new(self.clone())
    }
}

impl SessionTrait for ChunkTask {
    fn clone_as_session(&self) -> Box<dyn SessionTrait> {
        Box::new(self.clone())
    }

    fn session_id(&self) -> u32 {
        unimplemented!()
    }

    fn object_id(&self) -> ObjectId {
        unimplemented!()
    }

}

#[async_trait::async_trait]
impl TaskTrait for ChunkTask {

    async fn start(&self, _: Option<Box<dyn ToSourceTrait>>) {
        let text = match &self.0.encoder {
            ChunkEncodeDesc::Stream(range) => {

                let (offset, length) = 
                match ChunkEncodeDesc::create_stream(self.0.view.chunk()) {
                    ChunkEncodeDesc::Stream(chunk_range) => {
                        // if !chunk_range.contains(&range.start) {
                        //     error!("Out of range, except={:?}, start={}", chunk_range, range.start);
                        //     return;
                        // }
                        // if !chunk_range.contains(&range.end) {
                        //     error!("Out of range, except={:?}, end={}", chunk_range, range.end);
                        //     return;
                        // }
                        if !chunk_range.contains(range) {
                            error!("Out of range, except-range={}, got-range={}", chunk_range, range);
                            return;
                        }

                        if chunk_range.end == range.end {
                            (range.start as usize * PieceMessage::payload_max_len(), self.0.view.chunk().len())
                        } else {
                            (range.start as usize * PieceMessage::payload_max_len(), range.end as usize * PieceMessage::payload_max_len())
                        }
                    }
                };

                match self.0.view.read(offset, length).await {
                    Ok(v) => { v }
                    Err(err) => {
                        error!("failed read with err = {}", err);
                        return;
                    }
                }
                // let offset = *index as usize * PIECE_MESSAGE_PALOAD_MAX_LEN;
                // let mut v = vec![0u8; (*length) as usize];
                // let _ = self.0.view.read_to_end(&mut v, *offset as usize).await;
            }
        };

        if let Err(err) = self.0.manager.nds_stack().push_piece_data(&self.0.target, self.0.session_data, self.0.view.chunk(), self.0.encoder.clone(), text) {
            error!("failed push_piece_data with err = {}", err);
        }
    }

}


