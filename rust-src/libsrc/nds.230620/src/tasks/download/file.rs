
use std::{sync::{Arc, RwLock, atomic::{AtomicUsize, }}, collections::BTreeMap, };

use log::{info, error, debug};
use near_base::{file::FileObject, ChunkId, NearResult, ErrorCode, ObjectId, NearError, };

use crate::{nds_protocol::{PieceMessage, InterestMessage}, 
            tasks::{
                ToSourceTrait, manager::{TaskTrait, }, 
                DownloadSourceRef, SessionTrait
            }, MultiDownloadSource, inc::{ChunkWriterTrait, ChunkWriterFeedbackTrait}
        };

use super::{chunk::{ChunkTask, }, 
            h::{OnEventTrait, DownloadTaskTrait},
            DownloadManager, DownloadRequestTrait,
    };

// use super::{super::{{MultiDownloadSource}, },
//             DownloadManager, 
//             chunk::{ChunkTask, ChunkTaskWriterTrait}, OnEventTrait, DownloadTaskTrait, h::DownloadRequestTrait,
//     };
// use super::super::super::{inc::ChunkWriterTrait, };

// enum TaskStateImp {
//     None,
//     Prepair(PendingTaskStateRef),
//     Pending(PendingTaskStateRef),
//     #[allow(unused)]
//     Finished,
// }

type PendingTaskStateRef = Arc<PendingTaskState>;

struct PendingTaskState {
    index: AtomicUsize,
    tasks: BTreeMap<u32, ChunkTask>,
    tasks_ids: Vec<u32>,
    tasks_failure_array: RwLock<Vec<ChunkId>>,
}

impl PendingTaskState {
    fn new(all_tasks: Vec<ChunkTask>) -> Self {
        let (tasks, tasks_ids) = {
            let mut tasks_ids = vec![];
            let mut tasks = BTreeMap::new();

            for it in all_tasks.iter() {
                tasks.insert(it.session_id(), it.clone());
                tasks_ids.push(it.session_id());
            }
    
            (tasks, tasks_ids)
        };

        Self {
            index: AtomicUsize::new(0),
            tasks,
            tasks_ids,
            tasks_failure_array: RwLock::new(vec![]),
        }
    }

    pub fn append_failutre_task(&self, chunk: ChunkId) {
        self.tasks_failure_array.write().unwrap().push(chunk);
    }

    pub fn append_failutre_tasks(&self, chunks: Vec<ChunkId>) {
        let w = &mut *self.tasks_failure_array.write().unwrap();
        for chunk in chunks {
            w.push(chunk);
        }
    }

    pub(self) fn get_next_task(&self) -> Option<(usize, ChunkTask)> {
        let curr = self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        if curr >= self.tasks_ids.len() {
            info!("All tasks were exec.");
            None
        } else {
            if let Some(curr_index) = self.tasks_ids.get(curr) {
                self.tasks.get(curr_index)
                    .map(| task | {
                        (*curr_index as usize, task.clone())
                    })
            } else {
                unreachable!("fatal error.")
            }
        }
    }

}

#[async_trait::async_trait]
impl OnEventTrait for PendingTaskState {
    async fn on_piece_data(&self, data: &PieceMessage) -> NearResult<()> {
        debug!("session_id=: {}, chunk: {}, desc: {}, data-len: {}", data.session_data, data.chunk, data.desc, data.data.len());

        let task = 
            self.tasks.get(&data.session_data.session_sub_id)
                .ok_or_else(|| {
                    let error_message = format!("Not found session_sub_id {} ", data.session_data.session_sub_id);
                    error!("{}", error_message);
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_message)
                })?;

        task.on_piece_data(data).await
    }
}

struct FileTaskImpl {
    manager: DownloadManager,
    task_id: u32,
    file: FileObject,
    state: Option<PendingTaskStateRef>,
    // state: RwLock<TaskStateImp>,
    source: MultiDownloadSource,
}

#[derive(Clone)]
pub struct FileTask(Arc<FileTaskImpl>);

impl std::fmt::Display for FileTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileTask::{{session_id:{}, file:{}}}", self.session_id(), self.file().object_id())
    }
}

impl DownloadTaskTrait for FileTask {
    fn clone_as_downloadtask(&self) -> Box<dyn DownloadTaskTrait> {
        Box::new(self.clone())
    }
}

impl SessionTrait for FileTask {
    fn clone_as_session(&self) -> Box<dyn SessionTrait> {
        Box::new(self.clone())
    }

    fn session_id(&self) -> u32 {
        self.0.task_id
    }

    fn object_id(&self) -> ObjectId {
        self.0.file.object_id().clone()
    }

}

#[async_trait::async_trait]
impl TaskTrait for FileTask {

    async fn start(&self, _: Option<Box<dyn ToSourceTrait>>) {
        info!("{} begin...", self);

        let pending_state = self.0.state.clone().unwrap();

        {
            // for elf.0.source.source_count();
            let source_count = self.0.source.source_count();

            for _ in 0..source_count {
                match pending_state.get_next_task() {
                    Some((index, task)) => {
                        let source = match self.get_source(index) {
                            Ok(source) => { source },
                            Err(err) => {
                                error!("Failed to get_source with err: {}", err);
                                return;
                                // todo!: feedback
                            }
                        };

                        async_std::task::spawn(async move {
                                task.start(Some(source.to_source())).await;
                            });
                    }
                    None => {
                        info!("All task have been startup, the {} file's task waiting finished.", self.object_id());
                    }
                }
            }
        }

    }

}

#[async_trait::async_trait]
impl OnEventTrait for FileTask {
    async fn on_piece_data(&self, data: &PieceMessage) -> NearResult<()> {
        self.0.state.clone().unwrap().on_piece_data(data).await
    }
}

impl FileTask {
    pub fn new(manager: DownloadManager, file: FileObject, source: MultiDownloadSource) -> NearResult<Self> {
        let chunks = file.body().content().chunk_list().to_vec();

        if source.source_count() == 0 {
            error!("Failed to create file-task, because the source is none.");
            return Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not foudn download source."));
        }

        let task_id = manager.task_gen_id().generate().into_value();

        let ret = Self(Arc::new(FileTaskImpl{
            manager: manager.clone(),
            task_id,
            file,
            state: None,
            // chunks,
            // state: RwLock::new(TaskStateImp::None),
            source,
        }));


        let pending_task = {
            let mut arrays = vec![];
            let mut failure_tasks = vec![];

            for chunk in chunks.iter() {
                match ChunkTask::new(manager.clone(),
                                     chunk.clone(),
                                     Box::new(ret.clone()) as Box<dyn DownloadRequestTrait>,
                                     Box::new(ret.clone()) as Box<dyn ChunkWriterFeedbackTrait>,
                                     Some(Box::new(ret.clone()) as Box<dyn SessionTrait>)) {
                    Ok(task) => { arrays.push(task); }
                    Err(e) => {
                        error!("Failed to create chunk {} task with err = {}", chunk, e);
                        failure_tasks.push(chunk.clone());
                    }
                }
            }

            let state = PendingTaskState::new(arrays);
            state.append_failutre_tasks(failure_tasks);
            state
        };

        {
            let state = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut FileTaskImpl) };
            state.state = Some(Arc::new(pending_task));
        }

        Ok(ret)
    }

    pub(self) fn get_source(&self, index: usize) -> NearResult<DownloadSourceRef> {
        let cnt = self.0.source.source_count();

        if cnt == 0 {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_NO_AVAILABLE, "Source is empty"));
        }

        let mut index = index % cnt;
        let orig_index = index;

        let r = loop {
            let curr = self.0.source.source_of(index);
            if curr.is_enabled() {
                break(Some(curr));
            } else {
                index = index + 1;

                if index == orig_index {
                    break(None)
                } else if index == cnt {
                    index = 0;
                }
            }
        };

        r.ok_or(NearError::new(ErrorCode::NEAR_ERROR_NO_AVAILABLE, "No source available"))
    }

    pub fn file(&self) -> &FileObject {
        &self.0.file
    }

    pub fn manager(&self) -> &DownloadManager {
        &self.0.manager
    }
}

#[async_trait::async_trait]
impl ChunkWriterFeedbackTrait for FileTask {
    async fn finished(&self, _: Box<dyn ChunkWriterTrait>) {
        let pending_state = self.0.state.clone().unwrap();

        match pending_state.get_next_task() {
            Some((index, task)) => {
                let source = match self.get_source(index) {
                    Ok(source) => { source }
                    Err(err) => {
                        error!("failed to get_source with err: {}", err);
                        return;
                    }
                };

                async_std::task::spawn(async move {
                    task.start(Some(source.to_source())).await;
                });
            }
            None => {
                info!("All tasks are over.");
            }
        }
    }

    async fn err(&self, e: NearError) {

    }

}
// #[async_trait::async_trait]
// impl ChunkWriterTrait for FileTask {
//     fn clone_as_writer(&self) -> Box<dyn ChunkWriterTrait> {
//         Box::new(self.clone())
//     }

//     async fn write(&self, chunk: &ChunkId, offset: usize, content: &[u8]) -> NearResult<usize> {
//         Ok(0)
//     }

//     async fn err(&self, e: ErrorCode) -> NearResult<()> {
//         Ok(())
//     }

// }

#[async_trait::async_trait]
impl DownloadRequestTrait for FileTask {

    async fn interest_chunk(&self, target: DownloadSourceRef, chunk: &ChunkId, session: Option<Box<dyn SessionTrait>>) {
        self.manager().nds_stack().interest_chunk(target, chunk, session).await
    }

    async fn interest_chunk_v2(&self, target: DownloadSourceRef, object_id: Option<ObjectId>, message: InterestMessage) -> NearResult<()> {
        self.manager().nds_stack().interest_chunk_v2(target, object_id, message).await
    }
}
