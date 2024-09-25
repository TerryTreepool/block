
mod manager;
mod chunk;

pub use manager::Manager as UploadManager;

use near_base::{ChunkId, NearResult, ObjectId};
use crate::nds_protocol::ChunkPieceDesc;

pub trait UploadEventTrait: Send + Sync {
    fn push_piece_data(&self, target: &ObjectId, task_id: u32, chunk: &ChunkId, desc: ChunkPieceDesc, data: Vec<u8>) -> NearResult<()>;
}
