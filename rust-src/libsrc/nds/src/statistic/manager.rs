
use std::{sync::RwLock, collections::{BTreeMap, btree_map::Entry}};

use near_base::{ObjectId, file::FileObject, now};

use super::BytePerfStatPtr;

pub struct Manager {
    statistics: RwLock<BTreeMap<ObjectId, BytePerfStatPtr>>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            statistics: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn get(&self, file: &FileObject) -> BytePerfStatPtr {
        match   self.statistics
                    .read().unwrap()
                    .get(file.object_id()) {
            Some(stat) => stat.clone(),
            None => {
                let stat = BytePerfStatPtr::new(now(), file.desc().content().len());

                match   self.statistics
                            .write().unwrap()
                            .entry(file.object_id().clone()) {
                    Entry::Occupied(exist) => {
                        exist.get().clone()
                    }
                    Entry::Vacant(empty) => {
                        empty.insert(stat.clone());
                        stat
                    }
                }
            }
        }
    }
}
