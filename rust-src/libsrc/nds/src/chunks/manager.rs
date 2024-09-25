
use std::{sync::{Arc, RwLock}, collections::{BTreeMap, btree_map::Entry}, path::PathBuf};

use log::info;
use near_base::{ChunkId, NearResult, file::FileObject, ErrorCode};
use near_core::get_service_path;

use crate::inc::{ChunkReaderTrait, ChunkWriterTrait};

use super::{ChunkView, super::NdsStack, source::{ChunkFromTrack, ChunkFromCache}, source::SourceManager, store::MemChunk, ChunkAccess};

struct ManagerImpl {
    stack: NdsStack,
    views: RwLock<BTreeMap<ChunkId, ChunkView>>,
}

pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn new(stack: NdsStack) -> Self {
        Self(Arc::new(ManagerImpl{
            stack,
            views: RwLock::new(BTreeMap::new()),
        }))
    }

    pub fn nds_stack(&self) -> NdsStack {
        self.0.stack.clone()
    }

    pub fn view_of(&self, chunk: &ChunkId) -> Option<ChunkView> {
        self.0.views.read().unwrap().get(chunk).cloned()
    }

    /// create chunk view for downloader or uploader.
    pub async fn create_view(&self, chunk: &ChunkId, access: ChunkAccess) -> NearResult<ChunkView> {
        let view = {
            match self.view_of(chunk) {
                Some(view) => view,
                None => {
                    let content = match ChunkFromCache::from(chunk)
                                                    .path(&get_service_path(self.0.stack.service_name()))
                                                    .get_chunk()
                                                    .await {
                        Ok(content) => {
                            Ok(content)
                        }
                        Err(err) => {
                            match err.errno() {
                                ErrorCode::NEAR_ERROR_NOTFOUND => Ok(MemChunk::new(chunk.clone())),
                                _ => Err(err)
                            }
                        }
                    }?;

                    let view = match access {
                        ChunkAccess::Read => {
                            info!("Sucessful create_view for content = {}, access = readonly", content);
                            ChunkView::with_readonly(chunk.clone(), content.clone_as_reader())
                        }, 
                        ChunkAccess::Write => {
                            info!("Sucessful create_view for content = {}, access = write", content);
                            ChunkView::with_write(chunk.clone(), content)
                        },
                    };

                    match self.0.views.write().unwrap().entry(chunk.clone()) {
                        Entry::Occupied(mut existed) => {
                            existed.insert(view.clone());
                        }
                        Entry::Vacant(empty) => {
                            empty.insert(view.clone());
                        }
                    }

                    view
                }
            }
        };

        Ok(view)
    }

    /// track file
    pub async fn track_file(&self, file: &FileObject, path: &PathBuf) -> NearResult<()> {
        let track = ChunkFromTrack::open_with_file(path, file).await?;
        let views = &mut self.0.views.write().unwrap();
        let track_chunks = track.chunks();

        for chunk in track_chunks {
            views.insert(chunk.clone(), ChunkView::with_readonly(chunk.clone(), track.clone_as_reader()));
        }

        Ok(())
    }

}
