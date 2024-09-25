
use std::{sync::{Arc, RwLock}, };

use near_base::ObjectId;

mod upload;
mod download;
mod manager;

pub use manager::Manager;
pub use upload::UploadEventTrait;

pub struct DownloadSource {
    #[allow(unused)]
    target: ObjectId, 
    #[allow(unused)]
    referer: Option<String>
}

impl std::default::Default for DownloadSource {
    fn default() -> Self {
        Self {
            target: ObjectId::default(),
            referer: None,
        }
    }
}

impl DownloadSource {
    pub fn set_target(mut self, target: ObjectId) -> Self {
        self.target = target;
        self
    }

    pub fn set_referer(mut self, referer: Option<String>) -> Self {
        self.referer = referer;
        self
    }

    pub fn target(&self) -> &ObjectId {
        &self.target
    }

    pub fn referer(&self) -> Option<&str> {
        self.referer
            .as_ref()
            .map(|referer| referer.as_str())
    }
}

#[derive(Clone)]
pub struct SingleDownloadSource(Arc<DownloadSource>);

impl From<DownloadSource> for SingleDownloadSource {
    fn from(source: DownloadSource) -> Self {
        Self(Arc::new(source))
    }
}

impl AsRef<DownloadSource> for SingleDownloadSource {
    fn as_ref(&self) -> &DownloadSource {
        self.0.as_ref()
    }
}

#[derive(Clone)]
pub struct MultiDownloadSource(Arc<RwLock<Vec<SingleDownloadSource>>>);

impl MultiDownloadSource {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(Vec::new())))
    }

    pub fn add_source(self, source: SingleDownloadSource) -> Self {
        self.0.write().unwrap()
            .push(source);
        self
    }

    pub fn remove_source(&self, at: usize) -> Option<SingleDownloadSource> {
        let array = &mut *self.0.write().unwrap();

        if at >= array.len() {
            None
        } else {
            Some(array.remove(at))
        }
    }

}

#[derive(Debug, Clone)]
pub enum ChunkEncodeDesc {
    Unknown,
    Range(Option<u32>, Option<u32>, Option<i32>), 
} 

