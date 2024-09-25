
use std::{sync::{Arc, RwLock}, collections::{LinkedList}, };

use near_base::{NearResult, NearError, ObjectId, Sequence};
use near_core::near_error;

use crate::{NdsStack,
            tasks::{manager::{TaskTrait, }, }, 
            nds_protocol::{InterestMessage}
    };

use super::{chunk::ChunkTask, UploadTaskTrait};

struct DataImpl {
    // task_list: BTreeMap<u32, Box<dyn UploadTaskTrait>>,
    task_list: LinkedList<Box<dyn UploadTaskTrait>>,
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
                task_list: LinkedList::new(),
            })
        }))
    }

    pub async fn add_task(&self, target: ObjectId, message: &InterestMessage) -> NearResult<()> {
        let view = 
            self.nds_stack()
                .chunk_manager()
                .create_view(&message.chunk, crate::chunks::ChunkAccess::Read)
                .await
                .map_err(| e | {
                    near_error!(e.errno(), format!("failed to create_view chunk = {} with err = {}", &message.chunk, e))
                })?;

        let task = 
            ChunkTask::new(self.clone(), message.session_data, view, target, message.encoder.clone())
                .map_err(| e | {
                    near_error!(e.errno(), format!("Failed to ChunkTask::new({}) with err = {}", message.chunk, e))
                })?;

        self.0.data.write().unwrap()
            .task_list
            .push_back(task.clone_as_uploadtask());
        // match self.0.data.write().unwrap().task_list.entry(task.session_id()) {
        //     Entry::Occupied(_existed) => {
        //         unreachable!("Failed to add_task chunk = {} target = {}, because it was existed. But it isnot impositbale.", chunk, target);
        //     }
        //     Entry::Vacant(empty) => {
        //         empty.insert(task.clone_as_uploadtask());
        //     }
        // }

        async_std::task::spawn(async move {
            task.start(None).await
        });

        Ok(())
    }
}
