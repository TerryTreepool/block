
use std::sync::Arc;

use near_base::file::FileObject;

use super::{super::{{MultiDownloadSource}, manager::TaskTrait, },
            DownloadManager,
    };

struct FileTaskImpl {
    manager: DownloadManager,
    file: FileObject,
    source: MultiDownloadSource,
}

#[derive(Clone)]
pub struct FileTask(Arc<FileTaskImpl>);

impl std::fmt::Display for FileTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileTask::{{file:{}}}", self.file().object_id())
    }
}

#[async_trait::async_trait]
impl TaskTrait for FileTask {

    fn clone_as_task(&self) -> Box<dyn TaskTrait> {
        Box::new(self.clone())
    }

    async fn start(&self) {
    }

}

impl FileTask {
    pub fn new(manager: DownloadManager, file: FileObject, source: MultiDownloadSource, ) -> Self {
        let chunks = file.body().content().chunk_list();
        let ret = Self(Arc::new(FileTaskImpl{
            manager: manager.clone(),
            file,
            source,
        }));

        manager.add_task(ret.clone_as_task());

        ret
    }

    pub fn file(&self) -> &FileObject {
        &self.0.file
    }
}
