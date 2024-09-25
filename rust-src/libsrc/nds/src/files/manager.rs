
use std::{path::PathBuf, sync::{Arc, RwLock}, collections::BTreeMap};

use near_base::{file::FileObject, Timestamp, ObjectId, now, ErrorCode, NearResult, ChunkId};
use near_core::{path_utils::get_cache_path};

// use crate::nds::{ChunkReaderTrait, ChunkWriterTrait, NdsStack, };
use super::super::{{NdsStack, },
                    inc::{ChunkReaderTrait, ChunkWriterTrait},
    };

enum FileType {
    Picture(String /* suffix */),
    Audio(String /* suffix */),
    Video(String /* suffix */),
    Other(Option<String> /* suffix */),
}

enum FileSharePermission {
    Private,
    Family(Vec<u8>),
    Shared(Vec<u8>),
}

pub struct FileArticle {
    file: FileObject,
    create_timestamp: Timestamp,
    update_timestamp: Timestamp,
    file_type: FileType,
    permission: FileSharePermission,
}

#[derive(Clone)]
pub struct FileArticlePtr(Arc<FileArticle>);

impl FileArticlePtr {
    fn new(file: FileObject) -> Self {
        let now = now();

        Self( Arc::new( FileArticle {
            file,
            create_timestamp: now,
            update_timestamp: now,
            file_type: FileType::Other(None),
            permission: FileSharePermission::Private,
        }))
    }
}

struct ManagerImpl {
    stack: NdsStack,
    root_path: PathBuf,
    file_map: RwLock<BTreeMap<ObjectId, FileArticlePtr>>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn new(stack: NdsStack, root_path: Option<PathBuf>) -> Self {
        Self(Arc::new(ManagerImpl {
            stack,
            root_path: root_path.unwrap_or(get_cache_path()),
            file_map: RwLock::new(BTreeMap::new()),
        }))
    }

    pub fn article_of(&self, file_id: &ObjectId) -> Option<FileArticlePtr> {
        self.0.file_map.read().unwrap()
            .get(file_id)
            .map(| article | article.clone())
    }

    pub fn create_article(&self, file: FileObject) -> FileArticlePtr {
        let articles = &mut *self.0.file_map.write().unwrap();

        match articles.get(file.object_id()) {
            Some(article) => article.clone(),
            None => {
                let file_id = file.object_id().clone();
                let article = FileArticlePtr::new(file);
                let _ = articles.insert(file_id, article.clone());
                article
            }
        }
    }
}

#[async_trait::async_trait]
impl ChunkReaderTrait for Manager {
    fn clone_as_reader(&self) -> Box<dyn ChunkReaderTrait> {
        Box::new(self.clone())
    }

    async fn exists(&self, chunk: &ChunkId) -> bool {
        false
    }

    async fn get(&self, chunk: &ChunkId) -> NearResult<Vec<u8>> {
        unimplemented!()
    }
}

#[async_trait::async_trait]
impl ChunkWriterTrait for Manager {
    fn clone_as_writer(&self) -> Box<dyn ChunkWriterTrait> {
        Box::new(self.clone())
    }

    async fn write(&self, chunk: &ChunkId, content: Arc<Vec<u8>>) -> NearResult<()> {
        unimplemented!()

    }

    async fn finished(&self) -> NearResult<()> {
        unimplemented!()

    }

    async fn err(&self, e: ErrorCode) -> NearResult<()> {
        unimplemented!()

    }

}
