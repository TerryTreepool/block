
mod manager;
mod file;
mod chunk_list;
mod chunk;

pub use manager::{Manager as DownloadManager, Config as DownloadConfig};
pub use file::{FileTask as DownloadFileTask};
pub use chunk::{ChunkTask as DownloadChunkTask};

// pub trait OnEventTrait: Send + Sync {
//     fn interest(&self, _: Interest);
// }
