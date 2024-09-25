
mod manager;
mod chunk;

pub use manager::Manager as UploadManager;

use near_base::{ChunkId, NearResult, ObjectId};
use crate::nds_protocol::{PieceEncodeDesc, SessionData};

use super::manager::TaskTrait;

trait OnEventTrait: Send + Sync {
    fn push_piece_data(&self, target: &ObjectId, session_data: SessionData, chunk: &ChunkId, desc: PieceEncodeDesc, data: Vec<u8>) -> NearResult<()>;
}

pub trait UploadTaskTrait : TaskTrait {
    fn clone_as_uploadtask(&self) -> Box<dyn UploadTaskTrait>;
}

