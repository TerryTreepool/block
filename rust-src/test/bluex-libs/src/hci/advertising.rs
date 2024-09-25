
use std::sync::{atomic::AtomicBool, Arc};

use async_std::{task::JoinHandle, sync::RwLock};
use log::{error, info};

use near_base::{NearResult, queue::QueueGuard, NearError, ErrorCode};

use bluex::management::stream::Stream;

#[derive(Clone)]
pub struct AdvertisingProcessorData {
    data: Vec<u8>,
}

enum ProcessorStatus {
    Actived(Arc<ProcessorActiveStatus>),
    UnActived,
}

struct ProcessorActiveStatus {
    activing: AtomicBool,
    fut: Vec<JoinHandle<()>>,
}

pub struct AdvertisingProcessor {
    max_interval: u16,
    min_interval: u16,
    dataes: QueueGuard<AdvertisingProcessorData>,

    status: RwLock<ProcessorStatus>,
}

impl AdvertisingProcessor {
    fn new() -> Self {
        Self {
            min_interval: 1000,
            max_interval: 1000,
            dataes: QueueGuard::default(),
            status: RwLock::new(ProcessorStatus::UnActived),
        }
    }

    pub fn get_instance() -> &'static Self {
        static INSTANCE: once_cell::sync::OnceCell<AdvertisingProcessor> = once_cell::sync::OnceCell::new();

        INSTANCE.get_or_init(|| {
            let i = AdvertisingProcessor::new();
            i
        })
    }

    pub async fn active(&self) -> NearResult<()> {
        let w = &mut *self.status.write().await;

        match w {
            ProcessorStatus::Actived(_) => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_STARTUP, "already"))
            }
            ProcessorStatus::UnActived => {
                let status = Arc::new(ProcessorActiveStatus {
                    activing: AtomicBool::new(true),
                    fut: {
                        let fut = async_std::task::spawn(async move {
                            let _ = AdvertisingProcessor::get_instance().process().await;
                        });
                        vec![fut]
                    },
                });

                *w = ProcessorStatus::Actived(status);
                Ok(())
            }
        }
    }

    pub async fn wait_and_close(&self) {
        let w = &mut *self.status.write().await;

        match w {
            ProcessorStatus::UnActived => {}
            ProcessorStatus::Actived(status) => {
                // set activing
                status.activing.store(false, std::sync::atomic::Ordering::SeqCst);

                // wait fut
                let fut = {
                    let mut fut = vec![];
                    std::mem::swap(&mut fut, &mut unsafe { &mut *(Arc::as_ptr(status) as *mut ProcessorActiveStatus) }.fut);
                    fut
                };

                let _ = futures::future::join_all(fut).await;

                *w = ProcessorStatus::UnActived;
            }
        }
    }

    pub fn add_data(&self, data: Vec<u8>) -> NearResult<()> {
        self.dataes.push(AdvertisingProcessorData { data }, None);

        Ok(())
    }
}

impl AdvertisingProcessor {
    async fn process(&self) -> NearResult<()> {
        let activing = {
            if let ProcessorStatus::Actived(status) = &*self.status.read().await {
                status.clone()
            } else {
                unreachable!("fatal status");
            }
        };

        struct ProcessInner {
            stream: Stream,
        }

        impl std::ops::Drop for ProcessInner {
            fn drop(&mut self) {
                let _ = 
                    bluex::management::adverting::AdvertisingSwitch
                        ::close_advertising()
                        .cmd(&self.stream)
                        .map_err(| e | {
                            error!("failed open advertising-switch with err: {e}");
                            e
                        });
            }
        }

        let stream = 
            bluex::management::stream::Stream::open_default(Default::default())
                .map_err(| e | {
                    error!("failed open hci-socket with err: {e}");
                    e
                })?;

        let inner = ProcessInner{ stream };

        bluex::management::adverting::AdvertisingParam
            ::new(self.min_interval, self.max_interval)
                .cmd(&inner.stream)
                .map_err(| e | {
                    error!("failed set-advertising-parameter with err: {e}");
                    e
                })?;

        bluex::management::adverting::AdvertisingSwitch
            ::open_advertising()
                .cmd(&inner.stream)
                .map_err(| e | {
                    error!("failed open advertising-switch with err: {e}");
                    e
                })?;

        let mut timestamp = instant::Instant::now();

        loop {

            if !activing.activing.load(std::sync::atomic::Ordering::SeqCst) {
                info!("Preparing to end the advertising process.");
                break;
            }

            // get advertising data
            if let Some(data) = self.dataes.wait_and_take(std::time::Duration::from_millis(100)).await {

                let _ = std::mem::replace(&mut timestamp, instant::Instant::now());

                bluex::management::adverting::AdvertisingData
                    ::new(data.data)?
                    .cmd(&inner.stream)?;

            } else {
                if timestamp.elapsed() > std::time::Duration::from_secs(5) {
                    info!("AdvertisingProcessor quitting.");
                    *self.status.write().await = ProcessorStatus::UnActived;
                    break;
                }
            }
        }

        drop(inner);

        Ok(())

    }
}
