
use std::{sync::{RwLock, Arc}, collections::BTreeMap};

use near_base::{ChunkId, NearError, ErrorCode, NearResult};
use once_cell::sync::OnceCell;

use crate::chunks::store::MemChunk;

use super::{from_track::ChunkFromTrack, };

#[derive(Clone)]
enum SourceImpl {
    Track(Arc<ChunkFromTrack>),
}

struct ManagerImpl {
    chunks: RwLock<BTreeMap<ChunkId, SourceImpl>>,
}

pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    fn new() -> Self {
        Self(Arc::new(ManagerImpl{
            chunks: RwLock::new(BTreeMap::new()),
        }))
    }

    pub fn get_instance() -> &'static Self {
        static INSTANCE: OnceCell<Manager> = OnceCell::<Manager>::new();
        INSTANCE.get_or_init(|| {
            let m = Manager::new();
            m
        })
    }

    pub fn add_track(&self, track: ChunkFromTrack) {
        let track = Arc::new(track);
        let chunks = &mut *self.0.chunks.write().unwrap();

        for chunk in track.chunks() {
            chunks.insert(chunk.clone(), SourceImpl::Track(track.clone()));
        }
    }

    pub async fn get_chunk(&self, chunk: &ChunkId) -> NearResult<MemChunk> {
        let r = 
            self.0.chunks.read().unwrap()
                .get(chunk)
                .cloned()
                .ok_or(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found {} chunk.", chunk)))?;

        match r {
            SourceImpl::Track(track) => {
                track.get_chunk(chunk).await
            }
        }
    }

}


