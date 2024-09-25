
use std::sync::Arc;

use async_std::io::WriteExt;
use futures::AsyncReadExt;
use log::warn;
use near_base::{ChunkId, NearResult, NearError, ErrorCode};
use near_core::path_utils::get_cache_path;

use crate::nds::inc::{ChunkWriterTrait, ChunkReaderTrait};

use super::MemChunk;

struct Content {
    chunk: ChunkId,
}

#[derive(Clone)]
pub struct LocalChunk(Arc<Content>);

impl From<(ChunkId, MemChunk)> for LocalChunk {
    fn from(cx: (ChunkId, MemChunk)) -> Self {
        let (chunk, data) = cx;
        Self(Arc::new(Content{
            chunk,
        }))
    }
}

impl LocalChunk {
    async fn open(&self, chunk: ChunkId) {
        let len = chunk.len();

        let data = 
        match async_std::fs::OpenOptions::new()
            .create_new(false)
            .read(true)
            .open(get_cache_path().join(chunk.to_string()).as_path())
            .await {
            Ok(mut file) => {
                let mut data = vec![0u8; len];
                match file.read_to_end(&mut data)
                          .await {
                    Ok(size) => {
                        if size > len {
                            warn!("failed read chunk data with content size too long, size = {} chunk-len = {}", size, len);
                            None
                        } else if size < len {
                            warn!("failed read chunk data with content size not enough, size = {} chunk-len = {}", size, len);
                            None
                        } else {
                            Some(data)
                        }
                    }
                    Err(err) => {
                        warn!("failed read chunk data with err = {}", err);
                        None
                    }
                }
            }
            Err(err) => {
                warn!("failed open chunk data with err = {}", err);
                None
            }
        };

        Self(Arc::new(Content{
            chunk: chunk.clone(),
            data: data.map(|data|MemChunk::from((chunk, data)))
        }))
    }

    pub fn chunk(&self) -> &ChunkId {
        &self.0.chunk
    }
}

#[async_trait::async_trait]
impl ChunkReaderTrait for LocalChunk {
    fn clone_as_reader(&self) -> Box<dyn ChunkReaderTrait> {
        Box::new(self.clone())
    }

    async fn exists(&self, chunk: &ChunkId) -> bool {
        assert_eq!(chunk, self.chunk());

        match &self.0.data {
            Some(data) => data.exists(chunk).await,
            None => false,
        }
    }

    async fn get(&self, chunk: &ChunkId, offset: usize, length: usize) -> NearResult<&[u8]> {
        let r = match self.exists(chunk).await {
                true => {
                    let c = self.0.data
                        .as_ref()
                        .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("[{}] chunk block invalid", chunk)))?;
                    c.get(chunk, offset, length).await
                },
                false => { Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Could not found [{}] data", chunk))) }
        };

        r
    }

}

#[async_trait::async_trait]
impl ChunkWriterTrait for LocalChunk {
    fn clone_as_writer(&self) -> Box<dyn ChunkWriterTrait> {
        Box::new(self.clone())
    }

    async fn write(&self, chunk: &ChunkId) -> NearResult<()> {
        assert_eq!(&self.0.chunk, chunk);

        async_std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(get_cache_path().join(chunk.to_string()).as_path())
            .await
            .map_err(| err | {
                // near_error!(ErrorCode::NEAR_ERROR_SYSTERM, format!("failed create [{}] with err = {}, error-message = {}", chunk, err, stringify!(err)))
                err
            })?
            .write_all(self.0.data.as_ref())
            .await
            .map_err(| err | {
                println!("failed create [{}] with err {}", chunk, err);
                err
            })?;

        Ok(())
    }

    // async fn finished(&self) -> NearResult<()>;
    // async fn err(&self, e: ErrorCode) -> NearResult<()>;

}
