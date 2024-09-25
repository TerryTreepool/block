
use std::sync::Arc;

use log::debug;
use near_base::{file::FileObject, NearResult, ObjectId};

use crate::{NdsStack, MultiDownloadSource, nds_protocol::{PieceMessage, InterestMessage}};

use super::{SessionTrait, ToSourceTrait,
            download::{DownloadManager, DownloadConfig},
            upload::UploadManager,
        };

#[async_trait::async_trait]
pub trait TaskTrait: SessionTrait + Send + Sync {
    async fn start(&self, source: Option<Box<dyn ToSourceTrait>>);
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
    config: Config,
    downloads: Option<DownloadManager>,
    uploads: Option<UploadManager>,
    // reader: Box<dyn ChunkReaderTrait>,
    // writer: Box<dyn ChunkWriterTrait>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn new(stack: NdsStack, config: Option<Config>) -> NearResult<Self> {

        let ret = Self(Arc::new(ManagerImpl{
            config: config.unwrap_or(Config::default()),
            downloads: None,
            uploads: None,
        }));

        let downloads = DownloadManager::open(stack.clone(), ret.clone(), DownloadConfig{
            work_tasks: ret.0.config.work_count,
        })?;

        let uploads = UploadManager::open(stack.clone());

        let manager_impl = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut ManagerImpl ) };
        manager_impl.downloads = Some(downloads);
        manager_impl.uploads = Some(uploads);

        Ok(ret)
    }

    fn download_manager(&self) -> &DownloadManager {
        self.0.downloads.as_ref().unwrap()
    }

    fn upload_manager(&self) -> &UploadManager {
        self.0.uploads.as_ref().unwrap()
    }
}

impl Manager {
    pub async fn download_file(&self, file: FileObject, source: MultiDownloadSource) -> NearResult<()> {
        self.download_manager().download_file(file, source)
    }

    pub async fn upload(&self, target: ObjectId, message: &InterestMessage) -> NearResult<()> {
        debug!("prepairing interest to = {} on chunk = {}", target, &message.chunk);
        self.upload_manager().add_task(target, message).await
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

impl Manager {
    pub async fn on_piece_data(&self, data: &PieceMessage) -> NearResult<()> {
        self.download_manager().on_piece_data(data).await
    }
}
