
use std::{sync::{Arc, RwLock}, collections::BTreeMap};

use near_base::{ChunkId, NearResult};

use super::{ChunkView, super::NdsStack};

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
    pub async fn create_view(&self, chunk: ChunkId) -> NearResult<ChunkView> {
        let (view, newly) = {
            let views = &mut *self.0.views.write().unwrap();
            match views.get(&chunk) {
                Some(view) => (view.clone(), false),
                None => {
                    let view = ChunkView::new(chunk.clone());
                    views.insert(chunk, view.clone());
                    (view,  true)
                }
            }
        };

        if newly {
            // this is new view
            view.load().await?;
        }

        Ok(view)
    }
}
