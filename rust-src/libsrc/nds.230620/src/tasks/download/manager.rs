
use std::{sync::{Arc, RwLock}, time::Duration, collections::BTreeMap};

use async_std::task::JoinHandle;
use log::{info, };
use near_base::{NearResult, queue::Queue, file::FileObject, Sequence, ErrorCode, NearError};
use near_core::near_error;

use crate::{NdsStack, 
            nds_protocol::PieceMessage, 
            tasks::manager::Manager as TaskManager,
            MultiDownloadSource, };

use super::{DownloadFileTask,
            h::DownloadTaskTrait
        };

#[derive(Clone)]
pub struct Config {
    pub work_tasks: usize,
}

struct ManagerImpl {
    stack: NdsStack,
    task_gen_id: Sequence,
    parent: TaskManager,
    config: Config,
    task_array: Vec<JoinHandle<()>>,
    // queue: Queue<Box<dyn TaskTrait>>,
    queue: Queue<Box<dyn DownloadTaskTrait>>,
    working_array: RwLock<BTreeMap<u32 /* task-id */, Box<dyn DownloadTaskTrait>>>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn open(stack: NdsStack, parent: TaskManager, config: Config) -> NearResult<Self> {
        let ret = Self(Arc::new(ManagerImpl{
            stack,
            task_gen_id: Sequence::random(),
            parent,
            config: config.clone(),
            task_array: vec![],
            queue: Queue::default(),
            working_array: RwLock::new(BTreeMap::new()),
        }));

        let mut task_array = vec![];

        for _ in 0..config.work_tasks {
            let arc_ret = ret.clone();
            task_array.push(async_std::task::spawn(async move {
                arc_ret.run().await
            }));
        }

        let mut manager_impl = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut ManagerImpl) };
        manager_impl.task_array = task_array;

        Ok(ret)
    }

    pub async fn close(&self) {
        let mut task_array = vec![];

        unsafe { 
            std::ptr::copy(self.0.task_array.as_ptr(), 
                           task_array.as_mut_ptr(), 
                           self.0.task_array.len());
        }

        let _ = futures::future::join_all(task_array).await;

    }

    pub(super) fn nds_stack(&self) -> &NdsStack {
        &self.0.stack
    }

    pub(super) fn task_gen_id(&self) -> &Sequence {
        &self.0.task_gen_id
    }
}

impl Manager {
    pub fn download_file(&self, file: FileObject, source: MultiDownloadSource) -> NearResult<()> {
        let file_task = DownloadFileTask::new(self.clone(), file, source)?;

        self.0.queue.push(file_task.clone_as_downloadtask());

        Ok(())
    }

    async fn run(&self) {
        loop {
            if let Some(task) = self.0.queue.wait_and_take(Duration::from_secs(1)).await {
                let task = {
                    let w = &mut *self.0.working_array.write().unwrap();

                    match w.get(&task.session_id()) {
                        Some(_) => {
                            info!("The [{}] task has been running.", task.session_id());
                            continue;
                        }
                        None => {
                            w.insert(task.session_id(), task.clone_as_downloadtask());
                            task
                        }
                    }
                };

                task.start(None).await;
            } 
        }
    }
}

impl Manager {
    pub async fn on_piece_data(&self, data: &PieceMessage) -> NearResult<()> {
        let woring_chunk = {
            self.0.working_array.read().unwrap()
                .get(&data.session_data.session_id)
                .map(| task | task.clone_as_downloadtask())
                .ok_or_else(|| 
                    near_error!(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found session-id:{} in chunk:{} sync-piece", data.session_data.session_id, data.chunk))
                )
        }?;

        woring_chunk.on_piece_data(data).await
    }
}
