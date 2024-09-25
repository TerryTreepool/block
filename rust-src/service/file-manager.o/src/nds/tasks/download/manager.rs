
use std::sync::Arc;

use async_std::task::JoinHandle;
use near_base::{NearResult, queue::Queue};

use super::{super::Manager as TaskManager, 
            super::manager::TaskTrait, 
            file::FileTask,
    };

#[derive(Clone)]
pub struct Config {
    pub work_tasks: usize,
}

struct ManagerImpl {
    parent: TaskManager,
    config: Config,
    task_array: Vec<JoinHandle<()>>,
    queue: Queue<Box<dyn TaskTrait>>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn open(parent: TaskManager, config: Config) -> NearResult<Self> {
        let ret = Self(Arc::new(ManagerImpl{
            parent,
            config: config.clone(),
            task_array: vec![],
            queue: Queue::default(),
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

        unsafe { std::ptr::copy(self.0.task_array.as_ptr(), task_array.as_mut_ptr(), self.0.task_array.len()); }

        let _ = futures::future::join_all(task_array).await;

    }
}

impl Manager {
    pub fn add_task(&self, task: Box<dyn TaskTrait>) {
        self.0.queue.push(task);
    }

    async fn run(&self) {
        while let Some(task) = self.0.queue.take() {
            task.start().await;
        }
    }
}
