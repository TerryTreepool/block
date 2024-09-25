
use std::path::Path;

use near_base::{ChunkId, NearResult, ErrorCode, NearError, };

mod topic;

pub use topic::*;

#[async_trait::async_trait]
pub trait ChunkReaderTrait: Sync + Send {
    fn clone_as_reader(&self) -> Box<dyn ChunkReaderTrait>;

    async fn exists(&self, chunk: &ChunkId) -> bool;
    async fn get(&self, chunk: &ChunkId, offset: usize, content: &mut [u8]) -> NearResult<usize>;

}

#[async_trait::async_trait]
pub trait ChunkWriterTrait: SaveToPathTrait + Sync + Send {
    fn clone_as_writer(&self) -> Box<dyn ChunkWriterTrait>;

    async fn write(&self, chunk: &ChunkId, offset: usize, content: &[u8]) -> NearResult<usize>;
}

#[async_trait::async_trait]
pub trait ChunkWriterFeedbackTrait: Sync + Send {
    async fn finished(&self, writer: Box<dyn ChunkWriterTrait>);
    async fn err(&self, e: NearError);
}

#[async_trait::async_trait]
pub trait SaveToPathTrait: Sync + Send {
    async fn save_to_path(&self, path: &Path) -> NearResult<()>;
}

#[async_trait::async_trait]
pub trait LoadFromPathTrait: Sync + Send {
    type Target;
    async fn load_from_path(&self, path: &Path) -> NearResult<Self::Target>;
}
