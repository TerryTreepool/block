
use std::sync::{Arc, };

use near_base::{file::FileObject, NearResult, ChunkId, Sequence, ObjectId};

use super::super::NdsStack;
use super::MultiDownloadSource;
use super::{download::{DownloadManager, DownloadConfig, DownloadFileTask}};

#[async_trait::async_trait]
pub trait TaskTrait: Send + Sync {
    fn clone_as_task(&self) -> Box<dyn TaskTrait>;
    async fn start(&self);
}

pub struct Config {
    work_count: usize,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            work_count: num_cpus::get(),
        }
    }
}

struct ManagerImpl {
    stack: NdsStack,
    config: Config,
    task_gen_id: Sequence,
    // task_running: RwLock<BTreeMap<u32, ChunkView>>,
    downloads: Option<DownloadManager>,
    // reader: Box<dyn ChunkReaderTrait>,
    // writer: Box<dyn ChunkWriterTrait>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn new(stack: NdsStack, config: Option<Config>) -> NearResult<Self> {

        let ret = Self(Arc::new(ManagerImpl{
            stack: stack,
            config: config.unwrap_or(Config::default()),
            task_gen_id: Sequence::random(),
            // task_running: RwLock::new(BTreeMap::new()),
            downloads: None,
            // reader,
            // writer,
        }));

        let downloads = DownloadManager::open(ret.clone(), DownloadConfig{
            work_tasks: ret.0.config.work_count,
        })?;

        let manager_impl = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut ManagerImpl ) };
        manager_impl.downloads = Some(downloads);

        Ok(ret)
    }

    fn download_manager(&self) -> &DownloadManager {
        self.0.downloads.as_ref().unwrap()
    }
}

impl Manager {
    pub async fn download_file(&self, file: FileObject, source: MultiDownloadSource) -> NearResult<()> {
        DownloadFileTask::new(self.download_manager().clone(), file, source);
        Ok(())
    }

    pub async fn upload(&self, target: &ObjectId, chunk: ChunkId) -> NearResult<()> {
        Ok(())
    }
    // pub async fn start_upload(&self, task_id: SequenceValue, chunk: ChunkId, encode_codec: ChunkEncodeDesc, target: ObjectId, ) -> NearResult<()> {
    //     let task = 
    //         self.create_view(chunk)
    //             .start_upload(task_id, encode_codec, target)
    //             .map(| task | {
    //                 task.close_as_task()
    //             })
    //             .map_err(| err | {
    //                 // error!("failed start_upload() with err {}", err);
    //                 println!("failed start_upload() with err {}", err);
    //                 err
    //             })?;

    //     // start upload task

    //     Ok(())
    // }

}
