
use std::sync::Arc;

use near_base::{ChunkId, NearResult, };

mod topic;

pub use topic::*;

pub const CHUNK_MAX_SIZE: usize = 1 * 1024 * 1024;

#[async_trait::async_trait]
pub trait ChunkReaderTrait: Sync + Send {
    fn clone_as_reader(&self) -> Box<dyn ChunkReaderTrait>;

    async fn exists(&self, chunk: &ChunkId) -> bool;
    async fn get(&self, _: &ChunkId, offset: usize, length: usize) -> NearResult<&[u8]>;

}

#[async_trait::async_trait]
pub trait ChunkWriterTrait: Sync + Send {
    fn clone_as_writer(&self) -> Box<dyn ChunkWriterTrait>;

    async fn write(&self, chunk: &ChunkId, offset: usize, content: &[u8]) -> NearResult<()>;
    // async fn finished(&self) -> NearResult<()>;
    // async fn err(&self, e: ErrorCode) -> NearResult<()>;
}

