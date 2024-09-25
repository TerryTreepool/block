
use std::{sync::{RwLock, Arc}, collections::{BTreeMap, btree_map::Entry}, path::PathBuf, };

use near_base::{ChunkId, NearResult, ErrorCode, NearError, };
use near_core::path_utils::get_cache_path;
use once_cell::sync::OnceCell;

use crate::nds::inc::{ChunkReaderTrait, ChunkWriterTrait};

use super::{MemChunk};

struct DataImpl {
    data: Option<MemChunk>,
}

#[derive(Clone)]
struct Data(Arc<DataImpl>);

impl Data {
    fn as_mut(&mut self, chunk: &ChunkId) -> &mut MemChunk {
        let mut_data = unsafe { &mut *(Arc::as_ptr(&self.0) as *mut DataImpl) };

        if mut_data.data.is_none() {
            let  data = MemChunk::new(chunk.clone());
            mut_data.data = Some(data);
        }

        mut_data.data.as_mut().unwrap()
    }

}

#[async_trait::async_trait]
impl ChunkReaderTrait for Data {
    fn clone_as_reader(&self) -> Box<dyn ChunkReaderTrait> {
        Box::new(self.clone())
    }

    async fn exists(&self, _: &ChunkId) -> bool {
        self.0.data.is_some()
    }

    async fn get(&self, _: &ChunkId, offset: usize, length: usize) -> NearResult<&[u8]> {
        match self.0.data
                  .as_ref() {
            Some(data) => data.read(offset, length),
            None => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "not found"))
            }
        }
    }

}

struct ManagerImpl{
    root_path: PathBuf,
    chunks: RwLock<BTreeMap<ChunkId, Data>>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    fn new(root_path: PathBuf) -> Self {
        Self(Arc::new(ManagerImpl{
            root_path,
            chunks: RwLock::new(BTreeMap::new()),
        }))
    }

    pub fn get_instance() -> &'static Self {
        static INSTANCE: OnceCell<Manager> = OnceCell::new();
        INSTANCE.get_or_init(|| Self::new(get_cache_path()))
    }

    pub async fn chunk_of(&self, chunk: &ChunkId) -> NearResult<(Box<dyn ChunkReaderTrait>, Box<dyn ChunkWriterTrait>)> {
        let (newly, data) = {
            let chunks = &mut *self.0.chunks.write().unwrap();

            match chunks.entry(chunk.clone()) {
                Entry::Occupied(exist) => {
                    (false, exist.get().clone())
                }
                Entry::Vacant(not_found) => {
                    let data = Data(Arc::new(DataImpl { data: None }));
                    let _ = not_found.insert(data.clone());
                    (true, data)
                }
            }
        };

        if newly {
            let text = match MemChunk::load_from_file(chunk.clone(), Manager::get_chunk_path(&self.0.root_path, &chunk).as_path()).await {
                Ok(chunk_text) => Ok(Some(chunk_text)),
                Err(err) => {
                    match err.errno() {
                        ErrorCode::NEAR_ERROR_NOTFOUND => Ok(None),
                        _ => {
                            self.remove(&chunk);
                            Err(err)        
                        }
                    }
                }
            }?;

            let mut_data = unsafe {&mut * (Arc::as_ptr(&data.0) as *mut DataImpl)};
            mut_data.data = text;

            Ok((data.clone_as_reader(), self.clone_as_writer()))
        } else {
            Ok((data.clone_as_reader(), self.clone_as_writer()))
        }
    }

    pub fn remove(&self, chunk: &ChunkId) {
        let _ = self.0.chunks.write().unwrap().remove(chunk);
    }

    fn get_chunk_path(root_path: &PathBuf, chunk: &ChunkId) -> PathBuf {
        root_path.join(chunk.to_string())
    }
}

#[async_trait::async_trait]
impl ChunkWriterTrait for Manager {
    fn clone_as_writer(&self) -> Box<dyn ChunkWriterTrait> {
        Box::new(self.clone())
    }

    async fn write(&self, chunk: &ChunkId, offset: usize, content: &[u8]) -> NearResult<()> {
        match self.0.chunks.write().unwrap()
                  .get_mut(chunk) {
            Some(data) => {
                data.as_mut(chunk).write(offset, content)
            }
            None => Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found [{}] chunk", chunk)))
        }

    }
}