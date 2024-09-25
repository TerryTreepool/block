
mod manager;
mod file;
mod chunk;

pub use manager::{Manager as DownloadManager, Config as DownloadConfig};
pub use file::{FileTask as DownloadFileTask};
pub use chunk::{ChunkTask as DownloadChunkTask};

use near_base::NearResult;

use crate::nds_protocol::PieceMessage;

#[async_trait::async_trait]
pub trait OnEventTrait: Send + Sync {
    async fn on_piece_data(&self, data: &PieceMessage) -> NearResult<()>;
}

