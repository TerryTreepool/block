
use std::{collections::{BTreeMap, btree_map::Entry}, io::SeekFrom, path::PathBuf, sync::{Arc, RwLock}};

use async_std::{fs::File, io::{prelude::SeekExt, ReadExt}};
use near_base::{ChunkId, NearResult, file::FileObject, CHUNK_MAX_LEN, NearError, ErrorCode};
use near_core::near_error;
use near_util::ReadWithLimit;

use crate::inc::ChunkReaderTrait;

use super::super::store::MemChunk;

struct ChunkListDescImpl {
    total_len: u64,
    chunk_range_map: BTreeMap<ChunkId, (u64, u64)>,
    chunk_map: RwLock<BTreeMap<ChunkId, MemChunk>>,
}

impl From<&FileObject> for ChunkListDescImpl {
    fn from(file: &FileObject) -> Self {
        let mut chunks = BTreeMap::new();

        for (index, chunk) in file.body().content().chunk_list().iter().enumerate() {
            let chunk_len = chunk.len() as u64;
            let start = index as u64 * CHUNK_MAX_LEN;
            let end = start + chunk_len;
            chunks.insert(chunk.clone(), (start, end));
        }

        Self {
            total_len: file.desc().content().len(),
            chunk_range_map: chunks,
            chunk_map: RwLock::new(BTreeMap::new()),
        }
    }
}

impl ChunkListDescImpl {
    pub fn chunk_of(&self, chunk: &ChunkId) -> Option<&(u64, u64)> {
        self.chunk_range_map.get(chunk)
    }
}

struct ChunkFromTrackImpl {
    handle: File,
    chunk_desc: ChunkListDescImpl,

}

#[derive(Clone)]
pub struct ChunkFromTrack(Arc<ChunkFromTrackImpl>);

impl ChunkFromTrack {
    async fn open(path: &PathBuf, chunk_desc: ChunkListDescImpl) -> NearResult<Self> {
        let handle = async_std::fs::OpenOptions::new()
                    .create(false)
                    .read(true)
                    .open(path.as_path())
                    .await
                    .map_err(| err | {
                        near_error!(ErrorCode::NEAR_ERROR_SYSTERM, format!("failed open {} with err = {}, error-message = {}.", path.to_str().unwrap_or("None"), err, stringify!(err)))
                    })?;

        Ok(Self(Arc::new(ChunkFromTrackImpl {
            handle: handle,
            chunk_desc,
        })))
    }

    pub async fn open_with_file(path: &PathBuf, file: &FileObject) -> NearResult<Self> {
        ChunkFromTrack::open(path, ChunkListDescImpl::from(file)).await
    }
}

impl ChunkFromTrack {
    pub fn chunks(&self) -> Vec<&ChunkId> {
        let mut chunks = vec![];

        for (_, chunk) in self.0.chunk_desc.chunk_range_map.keys().enumerate() {
            chunks.push(chunk);
        }

        chunks
    }

    pub fn chunk_of(&self, chunk: &ChunkId) -> Option<MemChunk> {
        self.0.chunk_desc.chunk_map.read().unwrap().get(chunk).cloned()
    }
}

impl ChunkFromTrack {
    pub async fn get_chunk(&self, chunk: &ChunkId) -> NearResult<MemChunk> {
        let (start, end) = 
            self.0.chunk_desc
                .chunk_of(&chunk)
                .ok_or_else(|| {
                    near_error!(ErrorCode::NEAR_ERROR_NOTFOUND, format!("failed get_chunk({}) becaust it isn't file-list.", chunk))
                })?;

        let mut handle = self.0.handle.clone();
        let r = 
        handle.seek(SeekFrom::Start(*start))
            .await
            .map_err(| err | {
                near_error!(ErrorCode::NEAR_ERROR_SYSTERM, format!("failed seek {} to {} with err = {}, error-message = {}.", chunk, start, err, stringify!(err)))
            })?;

        let mut data = vec![0u8; (end - start) as usize];

        ReadWithLimit::new(*end, Box::new(handle))
            .read(data.as_mut_slice())
            .await
            .map_err(| err | {
                near_error!(ErrorCode::NEAR_ERROR_SYSTERM, format!("failed read {} chunk with err = {}, error-message = {}", chunk, err, stringify!(err)))
            })?;

        Ok(MemChunk::with_data(chunk.clone(), data))
    }

    pub async fn get_content(&self, chunk: &ChunkId) -> NearResult<Vec<u8>> {
        let (start, end) = 
            self.0.chunk_desc
                .chunk_of(&chunk)
                .ok_or_else(|| {
                    near_error!(ErrorCode::NEAR_ERROR_NOTFOUND, format!("failed get_chunk({}) becaust it isn't file-list.", chunk))
                })?;

        let mut handle = self.0.handle.clone();
        let r = 
        handle.seek(SeekFrom::Start(*start))
            .await
            .map_err(| err | {
                near_error!(ErrorCode::NEAR_ERROR_SYSTERM, format!("failed seek {} to {} with err = {}, error-message = {}.", chunk, start, err, stringify!(err)))
            })?;

        let mut data = vec![0u8; (end - start) as usize];

        ReadWithLimit::new(*end, Box::new(handle))
            .read(data.as_mut_slice())
            .await
            .map_err(| err | {
                near_error!(ErrorCode::NEAR_ERROR_SYSTERM, format!("failed read {} chunk with err = {}, error-message = {}", chunk, err, stringify!(err)))
            })?;

        Ok(data)
    }

}

#[async_trait::async_trait]
impl ChunkReaderTrait for ChunkFromTrack {
    fn clone_as_reader(&self) -> Box<dyn ChunkReaderTrait> {
        Box::new(self.clone())
    }

    async fn exists(&self, chunk: &ChunkId) -> bool {
        self.0.chunk_desc.chunk_of(chunk).is_some()
    }

    async fn get(&self, chunk: &ChunkId, offset: usize, content: &mut [u8]) -> NearResult<usize> {
        let chunk_ptr = match self.chunk_of(chunk) {
            Some(chunk_ptr) => { chunk_ptr },
            None => {
                let chunk_ptr = self.get_chunk(chunk).await?;

                match self.0.chunk_desc.chunk_map.write().unwrap().entry(chunk.clone()) {
                    Entry::Occupied(existed) => { existed.get().clone() }
                    Entry::Vacant(empty) => {
                        empty.insert(chunk_ptr.clone());
                        chunk_ptr
                    }
                }
            }
        };

        chunk_ptr.get(chunk, offset, content).await
    }

}
