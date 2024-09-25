
mod manager;
mod file;
mod chunk;
mod unsafe_array;
mod h;
mod encoder;

pub use manager::{Manager as DownloadManager, Config as DownloadConfig};
pub use file::{FileTask as DownloadFileTask};
pub use chunk::{ChunkTask as DownloadChunkTask};

use near_base::{ChunkId, ObjectId, NearResult};

use crate::nds_protocol::InterestMessage;

use super::{DownloadSourceRef, SessionTrait, };

#[async_trait::async_trait]
pub trait DownloadRequestTrait: Send + Sync {
    async fn interest_chunk(&self, target: DownloadSourceRef, chunk: &ChunkId, session: Option<Box<dyn SessionTrait>>);
    async fn interest_chunk_v2(&self, target: DownloadSourceRef, object_id: Option<ObjectId>, message: InterestMessage) -> NearResult<()>;
}
