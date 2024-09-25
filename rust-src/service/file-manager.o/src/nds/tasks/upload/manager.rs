
use std::{sync::{Arc, RwLock}, collections::BTreeMap};

use near_base::{ChunkId, NearResult};

use crate::nds::inc::ChunkReaderTrait;

use super::{super::super::NdsStack, };

struct DataImpl {
    // task_list: BTreeMap<ChunkId, >,
}

struct ManagerImpl {
    stack: NdsStack,
    data: RwLock<DataImpl>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn new(stack: NdsStack) -> Self {
        Self(Arc::new(ManagerImpl{
            stack,
            data: RwLock::new(DataImpl{
                // task_list: BTreeMap::new(),
            })
        }))
    }

    pub fn add_task(&self, chunk: ChunkId, reader: Box<dyn ChunkReaderTrait>) -> NearResult<()> {
        
        Ok(())
    }
}
