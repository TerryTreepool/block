
use std::{path::Path, io::ErrorKind, 
    };
use log::warn;

use async_std::io::{WriteExt, ReadExt};
use near_base::{ChunkId, NearResult, NearError, ErrorCode, };

use super::{super::{super::inc::*,
                    ChunkState,
                   },
    };

struct Content {
    chunk: ChunkId,
    data: Vec<u8>,

    // state: Ato
}

pub struct MemChunk(Content);

impl std::fmt::Display for MemChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemChunk: chunk={}, len={}", self.chunk(), self.0.data.len())
    }
}

impl MemChunk {
    pub fn new(chunk: ChunkId) -> Self {
        let data = vec![0u8; chunk.len()];

        Self(Content{
            chunk, 
            data,
        })
    }

    pub fn with_data(chunk: ChunkId, data: Vec<u8>) -> Self {
        Self(Content{
            chunk,
            data
        })
    }

    pub fn chunk(&self) -> &ChunkId {
        &self.0.chunk
    }

}

impl MemChunk {
    pub fn read_all(&self) -> &[u8] {
        self.0.data.as_slice()
    }

    pub fn read(&self, offset: usize, length: usize) -> NearResult<&[u8]> {
        let data = self.0.data.as_slice();
        let len = self.0.data.len();

        if offset > len {
            Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, format!("failed get context with expect offset, offset = {}, len = {}", offset, len)))
        } else {
            let will_get_size = offset + length;

            if will_get_size > len {
                Ok(&data[offset..len])
            } else {
                Ok(&data[offset..will_get_size])
            }
        }
    }

    pub fn write(&mut self, offset: usize, content: &[u8]) -> NearResult<()> {
        let content_len = content.len();
        let chunk_len = self.chunk().len();

        debug_assert!(offset > chunk_len, "offset exception, offset = {}, chunk-len = {}", offset, chunk_len);

        if offset + content_len > chunk_len {
            Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, format!("Could not write memory length exception, writing-len = {}, offset = {}, chunk-len = {}", content_len, offset, chunk_len)))
        } else {
            self.0.data[offset..].copy_from_slice(content);
            Ok(())
        }
    }

}

impl MemChunk {
    pub fn exists(path: &Path) -> bool {
        path.exists()    
    }

    pub async fn load_from_file(chunk: ChunkId, path: &Path) -> NearResult<MemChunk> {
        let len = chunk.len();

        let data = match async_std::fs::OpenOptions::new()
            .create_new(false)
            .read(true)
            .open(path)
            .await {
            Ok(mut file) => {
                let mut data = vec![0u8; len];
                match file.read_to_end(&mut data)
                          .await {
                    Ok(size) => {
                        if size > len {
                            let error_string = format!("failed read chunk data with content size too long, size = {} chunk-len = {}", size, len);
                            warn!("{}", error_string);
                            Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, error_string))
                        } else if size < len {
                            let error_string = format!("failed read chunk data with content size not enough, size = {} chunk-len = {}", size, len);
                            warn!("{}", error_string);
                            Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, error_string))
                        } else {
                            Ok(data)
                        }
                    }
                    Err(err) => {
                        let error_string = format!("failed read chunk data with err = {}", err);
                        warn!("{}", error_string);
                        Err(NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string))
                    }
                }
            }
            Err(err) => {
                let error_string = format!("failed open chunk data with err = {}", err);
                warn!("{}", error_string);
                match err.kind() {
                    ErrorKind::NotFound => {
                        Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string))
                    }
                    _ => {
                        Err(NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string))
                    }
                }
            }
        }?;

        Ok(Self::with_data(chunk, data))

    }

    pub async fn save_to_file(&self, path: &Path) -> NearResult<usize> {
        async_std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .await
            .map_err(| err | {
                println!("failed create [{:?}] with err {}", path.to_str(), err);
                err
            })?
            .write_all(self.read_all())
            .await
            .map_err(| err |{
                println!("failed write [{}] into [{:?}] with err {}", self.chunk(), path.to_str(), err);
                err
            })?;

        Ok(self.chunk().len())
    }

}

// #[async_trait::async_trait]
// impl ChunkWriterTrait for MemChunk {
//     fn clone_as_writer(&self) -> Box<dyn ChunkWriterTrait> {
//         Box::new(self.clone())
//     }

//     async fn write(&self, chunk: &ChunkId) -> NearResult<()> {
//         assert_eq!(self.chunk(), chunk);

//         LocalChunk::from((self.chunk().clone(), self.clone()))
//             .write(chunk)
//             .await
//     }

// }

// impl MemChunk {
//     pub fn new(chunk: ChunkId) -> Self {
//         Self(Arc::new(Content {
//             // ndc: NamedDataCache::clone(ndc), 
//             // chunks: RwLock::new(BTreeMap::new())
//         }))
//     }

//     pub async fn add(&self, id: ChunkId, chunk: Arc<Vec<u8>>) -> NearResult<()> {
//         let request = InsertChunkRequest {
//             chunk_id: id.to_owned(),
//             state: ChunkState::Ready,
//             ref_objects: None,
//             trans_sessions: None,
//             flags: 0,
//         };

//         let _ = self.0.ndc.insert_chunk(&request).await.map_err(|e| {
//             error!("record file chunk to ndc error! chunk={}, {}",id, e);
//             e
//         });

//         self.0.chunks.write().unwrap().insert(id, chunk);

//         Ok(())
//     }
// }


// #[async_trait]
// impl MemChunk for MemChunk {
//     fn clone_as_reader(&self) -> Box<dyn MemChunk> {
//         Box::new(self.clone())
//     }

//     async fn exists(&self, chunk: &ChunkId) -> bool {
//         self.0.chunks.read().unwrap().get(chunk).is_some()
//     }

//     async fn get(&self, chunk: &ChunkId) -> BuckyResult<Arc<Vec<u8>>> {
//         self.0.chunks.read().unwrap().get(chunk).cloned()
//             .ok_or_else(|| BuckyError::new(BuckyErrorCode::NotFound, "chunk not exists"))
//     }
// }


// #[async_trait]
// impl ChunkWriter for MemChunk {
//     fn clone_as_writer(&self) -> Box<dyn ChunkWriter> {
//         Box::new(self.clone())
//     }

//     async fn err(&self, _e: BuckyErrorCode) -> BuckyResult<()> {
//         Ok(())
//     }


//     async fn write(&self, chunk: &ChunkId, content: Arc<Vec<u8>>) -> BuckyResult<()> {
//         if chunk.len() == 0 {
//             return Ok(());
//         }

//         self.add(chunk.clone(), content).await
//     }

//     async fn finish(&self) -> BuckyResult<()> {
//         // do nothing
//         Ok(())
//     }
// }






