
use std::{sync::{Arc, }, 
    };

use near_base::{ChunkId, NearResult, NearError, ErrorCode};

use crate::nds::inc::ChunkReaderTrait;

struct Content {
    data: Vec<u8>,
}

#[derive(Clone)]
pub struct MemChunkStore(Arc<Content>);

impl std::fmt::Display for MemChunkStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MemChunkStore: len={}", self.0.data.len())
    }
}

impl From<Vec<u8>> for MemChunkStore {
    fn from(data: Vec<u8>) -> Self {
        Self(Arc::new(Content { data }))
    }
}

#[async_trait::async_trait]
impl ChunkReaderTrait for MemChunkStore {
    fn clone_as_reader(&self) -> Box<dyn ChunkReaderTrait> {
        Box::new(self.clone())
    }

    async fn exists(&self, _: &ChunkId) -> bool {
        true
    }

    async fn get(&self, _: &ChunkId, offset: usize, length: usize) -> NearResult<&[u8]> {
        let data = self.0.data.as_slice();
        let len = self.0.data.len();
        if offset > len {
            Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, format!("failed get context with expect offset, offset = {}, len = {}", offset, len)))
        } else {
            if offset + length > len {
                Ok(&data[offset..len])
            } else {
                Ok(&data[offset..offset+length])
            }
        }
    }

}

// impl MemChunkStore {
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
// impl ChunkReader for MemChunkStore {
//     fn clone_as_reader(&self) -> Box<dyn ChunkReader> {
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
// impl ChunkWriter for MemChunkStore {
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






