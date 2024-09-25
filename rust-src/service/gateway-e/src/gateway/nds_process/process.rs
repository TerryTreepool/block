
use std::{sync::{Arc, }, path::PathBuf};

use super::{NdsFileArticle, NdsManager};

struct NdsProcessImpl {
    nds_manager: NdsManager,
    nds_file: NdsFileArticle,
}

#[derive(Clone)]
pub struct NdsProcess(Arc<NdsProcessImpl>);

impl NdsProcess {
    pub fn new(nds_manager: NdsManager, nds_file: NdsFileArticle) -> Self {
        Self(Arc::new(NdsProcessImpl{
            nds_manager,
            nds_file,
        }))
    }

    #[inline]
    pub(crate) fn file_id(&self) -> &str {
        self.0.nds_file.file_id.as_str()
    }

    #[inline]
    pub(crate) fn file_path(&self) -> &PathBuf {
        &self.0.nds_file.file_path
    }

    #[inline]
    pub(self) fn nds_manager(&self) -> &NdsManager {
        &self.0.nds_manager
    }
}

impl NdsProcess {
    pub fn run(&self) {
        let arc_self = self.clone();
        async_std::task::spawn(async move {
                match arc_self.nds_manager()
                              .nds_stack()
                              .track_from_file(arc_self.file_path())
                              .await {
                Ok(_) => {
                    println!("create track and sync file = {} in nds-stack", arc_self.file_id());
                }
                Err(err) => {
                    println!("failed track file = {} in nds-stack with err = {}", arc_self.file_id(), err);
                }
            }
        });
    }
}
