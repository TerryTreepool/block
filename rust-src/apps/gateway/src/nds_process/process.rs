
use std::{sync::{Arc, RwLock}, path::PathBuf};

use super::NdsFileArticle;

struct NdsProcessImpl {
    nds_file: NdsFileArticle,
}

#[derive(Clone)]
pub struct NdsProcess(Arc<NdsProcessImpl>);

impl NdsProcess {
    pub fn new(nds_file: NdsFileArticle) -> Self {
        Self(Arc::new(NdsProcessImpl{
            nds_file,
        }))
    }

    pub(crate) fn file_id(&self) -> &str {
        self.0.nds_file.file_id.as_str()
    }

    pub(crate) fn file_path(&self) -> &PathBuf {
        &self.0.nds_file.file_path
    }
}

impl NdsProcess {
    pub fn run(&self) {

    }
}
