
use std::sync::{Arc, };

use near_base::{ObjectId, NearResult, };

use crate::{tasks::download::OnEventTrait, nds_protocol::PieceMessage};

use super::{super::{super::{chunks::ChunkView, 
                            nds_protocol::ChunkPieceDesc,
                    },
                    manager::{TaskTrait, },
            }, UploadEventTrait, 
    };

struct ChunkTaskImpl {
    task_id: u32,
    view: ChunkView,
    desc: ChunkPieceDesc,
    target: ObjectId,
    upload_cb: Box<dyn UploadEventTrait>,
}

#[derive(Clone)]
pub struct ChunkTask(Arc<ChunkTaskImpl>);

impl ChunkTask {
    pub fn new(task_id: u32, view: ChunkView, target: ObjectId, desc: ChunkPieceDesc, upload_cb: Box<dyn UploadEventTrait>) -> NearResult<Self> {
        Ok(Self(Arc::new(ChunkTaskImpl{
            task_id,
            view, 
            desc,
            target,
            upload_cb,
        })))
    }

}

#[async_trait::async_trait]
impl TaskTrait for ChunkTask {
    fn task_id(&self) -> u32 {
        self.0.task_id
    }

    fn clone_as_task(&self) -> Box<dyn TaskTrait> {
        Box::new(self.clone())
    }

    async fn start(&self) {
        let text = match &self.0.desc {
            ChunkPieceDesc::Range(offset, length) => {
                let mut v = vec![0u8; (*length) as usize];
                let _ = self.0.view.read_to_end(&mut v, *offset as usize);
                v
            }
        };

        if let Err(err) = self.0.upload_cb.push_piece_data(&self.0.target, self.0.task_id, self.0.view.chunk(), self.0.desc.clone(), text) {
            println!("failed push_piece_data with err = {}", err);
        }
    }

}

#[async_trait::async_trait]
impl OnEventTrait for ChunkTask {
    async fn on_piece_data(&self, _data: &PieceMessage) -> NearResult<()> {
        unreachable!("FATAL: I'm Upload's Task. Why Download's Task appeared.")
    }
}
