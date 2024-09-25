
use std::{sync::{Arc, RwLock, atomic::AtomicBool}, };

use near_base::{ObjectId, };

mod upload;
mod download;
mod manager;

pub use manager::Manager;
pub use download::DownloadRequestTrait;

pub trait SessionTrait: Send + Sync {
    fn clone_as_session(&self) -> Box<dyn SessionTrait>;
    fn session_id(&self) -> u32;
    fn object_id(&self) -> ObjectId;
}

pub struct DownloadSource {
    #[allow(unused)]
    target: ObjectId, 
    #[allow(unused)]
    referer: Option<String>,
    #[allow(unused)]
    enabled: AtomicBool,
}

impl std::default::Default for DownloadSource {
    fn default() -> Self {
        Self {
            target: ObjectId::default(),
            referer: None,
            enabled: AtomicBool::new(true),
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

    pub fn enabled(&mut self) {
        self.enabled.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn disabled(&mut self) {
        self.enabled.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(std::sync::atomic::Ordering::SeqCst)
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

pub type DownloadSourceRef = Arc<DownloadSource>;

pub trait ToSourceTrait: Send + Sync {
    fn to_source(&self) -> Box<dyn ToSourceTrait>;
    fn source_count(&self) -> usize;
    fn source_of(&self, index: usize) -> DownloadSourceRef;
}

impl ToSourceTrait for DownloadSourceRef {
    fn to_source(&self) -> Box<dyn ToSourceTrait> {
        Box::new(self.clone())
    }

    fn source_count(&self) -> usize {
        1
    }

    fn source_of(&self, _: usize) -> DownloadSourceRef {
        self.clone()
    }
}

#[derive(Clone)]
pub struct SingleDownloadSource(DownloadSourceRef);

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

impl ToSourceTrait for SingleDownloadSource {
    fn to_source(&self) -> Box<dyn ToSourceTrait> {
        self.0.to_source()
    }

    fn source_count(&self) -> usize {
        self.0.source_count()
    }

    fn source_of(&self, index: usize) -> DownloadSourceRef {
        self.0.source_of(index)
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

    pub fn source(&self) -> Vec<SingleDownloadSource> {
        self.0.read().unwrap()
            .iter()
            .filter(| item | {
                item.0.is_enabled()
            })
            .cloned()
            .collect()
    }

}

impl ToSourceTrait for MultiDownloadSource {
    fn to_source(&self) -> Box<dyn ToSourceTrait> {
        Box::new(self.clone())
    }

    fn source_count(&self) -> usize {
        self.0.read().unwrap().len()
    }

    fn source_of(&self, index: usize) -> DownloadSourceRef {
        let r = &*self.0.read().unwrap();
        debug_assert!((index < r.len()));
        r.get(index).unwrap().source_of(0)
    }
}
