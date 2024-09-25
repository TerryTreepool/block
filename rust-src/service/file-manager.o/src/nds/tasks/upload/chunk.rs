
use std::sync::{Arc, };

use near_base::{ObjectId};

use super::{super::{super::{chunks::ChunkView, 
                            nds_protocol::ChunkPieceDesc,
                    },
                    manager::{TaskTrait, },
                    ChunkEncodeDesc,
                    
            }, UploadEventTrait, 
    };

struct ChunkSession {
}

struct ChunkTaskImpl {
    view: ChunkView,
    desc: ChunkPieceDesc,
    target: ObjectId,
    upload_cb: Box<dyn UploadEventTrait>,
}

#[derive(Clone)]
pub struct ChunkTask(Arc<ChunkTaskImpl>);

impl ChunkTask {
    pub fn new(view: ChunkView, target: ObjectId, desc: ChunkPieceDesc, upload_cb: Box<dyn UploadEventTrait>) -> Self {
        Self(Arc::new(ChunkTaskImpl{
            view, 
            desc,
            target,
            upload_cb,
        }))
    }

}

#[async_trait::async_trait]
impl TaskTrait for ChunkTask {
    fn clone_as_task(&self) -> Box<dyn TaskTrait> {
        Box::new(self.clone())
    }

    async fn start(&self) {
        let arc_self = self.clone();

        let text = match &self.0.desc {
            ChunkPieceDesc::Range(offset, length) => {
                let mut v = vec![0u8; (*length) as usize];
                let _ = self.0.view.read_to_end(&mut v, *offset as usize, *length as usize).await;
                v
            }
        };

        if let Err(err) = self.0.upload_cb.push_piece_data(self.0.target.clone(), self.0.view.chunk().clone(), self.0.desc.clone(), text) {
            println!("failed push_piece_data with err = {}", err);
        }
    }

}
