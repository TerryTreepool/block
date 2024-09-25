
use near_base::{ChunkId, NearResult};

use super::{ChunkReader, ChunkWriter};

pub struct Builder<'a> {
    chunk: &'a ChunkId,
}

impl<'a> Builder<'a> {
    pub fn new(chunk: &'a ChunkId) -> Self {
        Self{
            chunk
        }
    }

    pub fn build(self) -> NearResult<(ChunkReader, ChunkWriter)> {
        unimplemented!()
    }
}
