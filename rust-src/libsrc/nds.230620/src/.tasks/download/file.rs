
use std::sync::{Arc, RwLock, atomic::AtomicUsize};

use log::{info, error};
use near_base::{file::FileObject, ChunkId, NearResult, ErrorCode, ObjectId, NearError};

use crate::nds_protocol::PieceMessage;

use super::{super::{{SingleDownloadSource, MultiDownloadSource}, manager::TaskTrait, },
            DownloadManager, 
            chunk::{ChunkTask, ChunkTaskWriterTrait}, OnEventTrait,
    };
use super::super::super::{inc::ChunkWriterTrait, };

enum TaskStateImp {
    Prepair(PendingTaskStateRef),
    Pending(PendingTaskStateRef),
    #[allow(unused)]
    Finished,
}

type PendingTaskStateRef = Arc<PendingTaskState>;

struct PendingTaskState {
    index: AtomicUsize,
    pending_tasks: Vec<ChunkTask>,
    working_tasks: Vec<ChunkTask>,
}

struct FileTaskImpl {
    manager: DownloadManager,
    task_id: u32,
    file: FileObject,
    chunks: Vec<ChunkId>,
    state: RwLock<TaskStateImp>,
    // source_array: Vec<SingleDownloadSource>,
}

#[derive(Clone)]
pub struct FileTask(Arc<FileTaskImpl>);

impl std::fmt::Display for FileTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileTask::{{task_id:{}, file:{}}}", self.task_id(), self.file().object_id())
    }
}

#[async_trait::async_trait]
impl TaskTrait for FileTask {

    fn clone_as_task(&self) -> Box<dyn TaskTrait> {
        Box::new(self.clone())
    }

    fn task_id(&self) -> u32 {
        self.0.task_id
    }

    async fn start(&self) {
        info!("{} begin...", self);

        // let pending_state = {
        //     let state = &mut *self.0.state.write().unwrap();
        //     match state {
        //         TaskStateImp::Prepair => {
        //             let pending_state = Arc::new(PendingTaskState{
        //                 index: AtomicUsize::new(0),
        //                 pending_tasks: {
        //                     let source = &self.0.source_array;
        //                     let source_cnt = source.len();

        //                     let mut arrays = vec![];
        //                     for (chunk_index, chunk) in self.0.chunks.iter().enumerate() {
        //                         match ChunkTask::new(self.manager().clone(), 
        //                                              chunk.clone(),
        //                                              source.get(chunk_index % source_cnt).unwrap().clone(),
        //                                              Box::new(self.clone()) as Box<dyn ChunkTaskWriterTrait>) {
        //                             Ok(task) => { arrays.push(task); }
        //                             Err(e) => {
        //                                 error!("Failed to create chunk {} task with err = {}", chunk, e);
        //                             }
        //                         }
        //                     }
        //                     arrays
        //                 }
        //             });

        //             *state = TaskStateImp::Pending(pending_state.clone());
        //             pending_state
        //         }
        //         TaskStateImp::Pending(_) => {
        //             info!("Task has been startup...");
        //             return;
        //         }
        //         TaskStateImp::Finished => {
        //             info!("Task has been finished...");
        //             return;
        //         }
        //     }
        // };

        // {
        //     let source = &self.0.source_array;
        //     let source_cnt = source.len();
        //     let index = &pending_state.index;
        //     let tasks = &pending_state.pending_tasks;

        //     for _ in 0..source_cnt {
        //         let curr = index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        //         let task = tasks.get(curr % source_cnt).unwrap().clone();

        //         async_std::task::spawn(async move {
        //             task.start().await;
        //         });
        //     }
        // }
    }

}

#[async_trait::async_trait]
impl OnEventTrait for FileTask {
    async fn on_piece_data(&self, data: &PieceMessage) -> NearResult<()> {
        Ok(())
    }
}

impl FileTask {
    pub fn new(manager: DownloadManager, file: FileObject, ) -> NearResult<Self> {
        let source_array = source.source();
        let chunks = file.body().content().chunk_list().to_vec();

        if source_array.len() == 0 {
            error!("Failed to create file-task, because the source is none.");
            return Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not foudn download source."));
        }

        let task_id = manager.task_gen_id().generate().into_value();

        {
            let source = source_array/* &self.0.source_array */;
            let source_cnt = source.len();

            let mut arrays = vec![];
            for (chunk_index, chunk) in self.0.chunks.iter().enumerate() {
                match ChunkTask::new(self.manager().clone(), 
                                        chunk.clone(),
                                        source.get(chunk_index % source_cnt).unwrap().clone(),
                                        Box::new(self.clone()) as Box<dyn ChunkTaskWriterTrait>) {
                    Ok(task) => { arrays.push(task); }
                    Err(e) => {
                        error!("Failed to create chunk {} task with err = {}", chunk, e);
                    }
                }
            }
            arrays
        }
        // let pending_state = {
        //     let state = &mut *self.0.state.write().unwrap();
        //     match state {
        //         TaskStateImp::Prepair => {
        //             let pending_state = Arc::new(PendingTaskState{
        //                 index: AtomicUsize::new(0),
        //                 pending_tasks: {
        //                     let source = &self.0.source_array;
        //                     let source_cnt = source.len();

        //                     let mut arrays = vec![];
        //                     for (chunk_index, chunk) in self.0.chunks.iter().enumerate() {
        //                         match ChunkTask::new(self.manager().clone(), 
        //                                              chunk.clone(),
        //                                              source.get(chunk_index % source_cnt).unwrap().clone(),
        //                                              Box::new(self.clone()) as Box<dyn ChunkTaskWriterTrait>) {
        //                             Ok(task) => { arrays.push(task); }
        //                             Err(e) => {
        //                                 error!("Failed to create chunk {} task with err = {}", chunk, e);
        //                             }
        //                         }
        //                     }
        //                     arrays
        //                 }
        //             });

        //             *state = TaskStateImp::Pending(pending_state.clone());
        //             pending_state
        //         }
        //         TaskStateImp::Pending(_) => {
        //             info!("Task has been startup...");
        //             return;
        //         }
        //         TaskStateImp::Finished => {
        //             info!("Task has been finished...");
        //             return;
        //         }
        //     }
        // };

        let ret = Self(Arc::new(FileTaskImpl{
            manager,
            task_id,
            file,
            chunks,
            state: RwLock::new(TaskStateImp::Prepair),
            source_array: source_array,
        }));

        Ok(ret)
    }

    pub fn file(&self) -> &FileObject {
        &self.0.file
    }

    pub fn manager(&self) -> &DownloadManager {
        &self.0.manager
    }
}

impl ChunkTaskWriterTrait for FileTask {
    fn file_id(&self) -> &ObjectId {
        self.0.file.object_id()
    }
}

#[async_trait::async_trait]
impl ChunkWriterTrait for FileTask {
    fn clone_as_writer(&self) -> Box<dyn ChunkWriterTrait> {
        Box::new(self.clone())
    }

    async fn write(&self, chunk: &ChunkId, offset: usize, content: &[u8]) -> NearResult<usize> {
        Ok(0)
    }

    async fn err(&self, e: ErrorCode) -> NearResult<()> {
        Ok(())
    }

}
