
use std::{path::Path, io::ErrorKind, sync::Arc,
    };
use log::warn;

use async_std::io::{WriteExt, ReadExt};
use near_base::{ChunkId, NearResult, NearError, ErrorCode, FileEncoder, };
use near_core::near_error;

use crate::inc::{ChunkReaderTrait, ChunkWriterTrait, SaveToPathTrait, LoadFromPathTrait};

// use super::{super::{super::inc::*,
//                     ChunkState,
//                    },
//     };

struct Content {
    chunk: ChunkId,
    data: Vec<u8>,
}

#[derive(Clone)]
pub struct MemChunk(Arc<Content>);

impl std::fmt::Display for MemChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemChunk: chunk={}, chunk-len={}, flight-size={}", self.chunk(), self.chunk().len(), self.0.data.len())
    }
}

impl MemChunk {
    pub fn new(chunk: ChunkId) -> Self {
        let data = vec![0u8; chunk.len()];
        Self::with_data(chunk, data)
    }

    pub fn with_data(chunk: ChunkId, data: Vec<u8>) -> Self {
        Self(Arc::new(Content{
            chunk,
            data
        }))
    }

    #[inline]
    pub fn chunk(&self) -> &ChunkId {
        &self.0.chunk
    }

    #[inline]
    pub fn finished(&self) -> bool {
        self.0.data.len() == self.chunk().len()
    }

    #[inline]
    pub fn flightsize(&self) -> usize {
        self.0.data.len()
    }
}

impl AsRef<[u8]> for MemChunk {
    fn as_ref(&self) -> &[u8] {
        self.0.data.as_slice()
    }
}

impl MemChunk {

    pub fn read_to_end(&self, offset: usize, length: usize) -> NearResult<&[u8]> {
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

    pub fn write_all(&self, offset: usize, content: &[u8]) -> NearResult<usize> {
        let content_len = content.len();
        let chunk_len = self.chunk().len();

        debug_assert!(offset <= chunk_len, "offset exception, offset = {}, chunk-len = {}", offset, chunk_len);

        if offset + content_len > chunk_len {
            Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, format!("Could not write memory length exception, writing-len = {}, offset = {}, chunk-len = {}", content_len, offset, chunk_len)))
        } else {
            unsafe {
                let mut_self = &mut * (Arc::as_ptr(&self.0) as *mut Content);

                std::ptr::copy(content.as_ptr(), (&mut mut_self.data[offset..]).as_mut_ptr(), content_len);
            }
            Ok(content_len)
        }
    }

}

impl MemChunk {
    pub fn exists(path: &Path) -> bool {
        path.exists()
    }

}

#[async_trait::async_trait]
impl ChunkReaderTrait for MemChunk {
    fn clone_as_reader(&self) -> Box<dyn ChunkReaderTrait> {
        Box::new(self.clone())
    }

    async fn exists(&self, chunk: &ChunkId) -> bool {
        self.chunk() == chunk
    }

    async fn get(&self, chunk: &ChunkId, offset: usize, content: &mut [u8]) -> NearResult<usize> {
        if self.chunk() == chunk {
            self.read_to_end(offset, content.len())
                .map(| text | {
                    content.copy_from_slice(text);
                    text.len()
                })
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, format!("Failed to ChunkReaderTrait::get() execpt = {} got = {}", self.chunk(), chunk)))
        }
    }
}

#[async_trait::async_trait]
impl ChunkWriterTrait for MemChunk {
    fn clone_as_writer(&self) -> Box<dyn ChunkWriterTrait> {
        Box::new(self.clone())
    }

    async fn write(&self, chunk: &ChunkId, offset: usize, content: &[u8]) -> NearResult<usize> {
        if self.chunk() == chunk {
            self.write_all(offset, content)
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, format!("Failed to ChunkReaderTrait::get() execpt = {} got = {}", self.chunk(), chunk)))
        }
    }

}

#[async_trait::async_trait]
impl SaveToPathTrait for MemChunk {
    async fn save_to_path(&self, path: &Path) -> NearResult<()> {
        println!("{}", path.display());
        let chunk_path = path.join(self.chunk().to_string58());

        let mut file = async_std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(chunk_path.as_path())
            .await
            .map_err(| err | {
                near_error!(ErrorCode::NEAR_ERROR_SYSTERM, format!("failed create [{:?}] with err = {}, error-message = {}", chunk_path.display(), err, stringify!(err)))
            })?;

        file.write_all(self.as_ref())
        .await
        .map_err(| err |{
            near_error!(ErrorCode::NEAR_ERROR_SYSTERM, format!("failed write [{}] into [{:?}] with err = {}, error-message = {}", self.chunk(), path.display(), err, stringify!(err)))
        })?;
        
        let _ = file.flush().await;

        Ok(())
    }
}

pub struct MemChunkBuilder<'a> {
    chunk: &'a ChunkId,
}

impl<'a> MemChunkBuilder<'a> {
    pub fn new(chunk: &'a ChunkId) -> Self {
        Self {
            chunk
        }
    }
}

#[async_trait::async_trait]
impl LoadFromPathTrait for MemChunkBuilder<'_> {
    type Target = MemChunk;

    async fn load_from_path(&self, path: &Path) -> NearResult<Self::Target> {
        let chunk_path = path.join(self.chunk.to_string());
        let len = self.chunk.len();

        let data = match async_std::fs::OpenOptions::new()
            .create_new(false)
            .read(true)
            .open(chunk_path)
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

        Ok(Self::Target::with_data(self.chunk.clone(), data))
    }
}

#[test]
fn test_mem_chunk() {
    use near_base::hash_data;
    async_std::task::block_on(async move {
        let hash = hash_data("1234567890".as_bytes());
        let chunk_id = ChunkId::from((hash, 10));
    
        let mem_chunk = MemChunk::new(chunk_id.clone());
    
        mem_chunk.write(&chunk_id, 0, "1234567890".as_bytes()).await.unwrap();

        println!("mem_chunk: {}", mem_chunk);
            
    });
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






