
use std::{sync::{Arc, RwLock}, collections::{BTreeMap, btree_map::Entry}, };

use log::warn;
use near_base::{ChunkId, NearResult, NearError, ObjectId, Sequence};
use near_core::near_error;

use crate::{NdsStack,
            tasks::manager::{TaskTrait, }, 
            nds_protocol::ChunkPieceDesc
    };

use super::chunk::ChunkTask;

struct DataImpl {
    task_list: BTreeMap<u32, Box<dyn TaskTrait>>,
}

struct ManagerImpl {
    stack: NdsStack,
    task_gen_id: Sequence,
    data: RwLock<DataImpl>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn nds_stack(&self) -> &NdsStack {
        &self.0.stack
    }

    pub(self) fn task_gen_id(&self) -> &Sequence {
        &self.0.task_gen_id
    }
}

impl Manager {
    pub fn open(stack: NdsStack) -> Self {
        Self(Arc::new(ManagerImpl{
            stack,
            task_gen_id: Sequence::random(),
            data: RwLock::new(DataImpl{
                task_list: BTreeMap::new(),
            })
        }))
    }

    pub async fn add_task(&self, target: ObjectId, chunk: ChunkId, desc: ChunkPieceDesc) -> NearResult<()> {
        let view = 
            self.nds_stack()
                .chunk_manager()
                .create_view(&chunk, crate::chunks::ChunkAccess::Read)
                .await
                .map_err(| e | {
                    near_error!(e.errno(), format!("failed to create_view chunk = {} with err = {}", chunk, e))
                })?;

        let task = 
            ChunkTask::new(self.task_gen_id().generate().into_value(), view, target, desc, Box::new(self.nds_stack().clone()))
                .map_err(| e | {
                    near_error!(e.errno(), format!("Failed to ChunkTask::new({}) with err = {}", chunk, e))
                })?;

        match self.0.data.write().unwrap().task_list.entry(task.task_id().clone()) {
            Entry::Occupied(_existed) => {
                warn!("Failed to add_task chunk = {} target = {}, because it was existed.", chunk, target);
            }
            Entry::Vacant(empty) => {
                empty.insert(task.clone_as_task());
            }
        }

        async_std::task::spawn(async move {

            // todo!
            task.start().await
        });

        Ok(())
    }
}
