
#[cfg(any(target_os = "android", target_os = "ios"))]
pub mod bridge_generated;

use std::{sync::{atomic::AtomicBool, Arc, Mutex, RwLock}, time::Duration};

use api::Result;
use async_std::{future, task::JoinHandle};
use bluex::management::scaner::ScanResult;

use hci::{advertising, scanning::ScanProcessorEventTrait};
use once_cell::sync::OnceCell;

mod api;
mod hci;

#[derive(Default)]
struct ScanResultsImp {
    running: AtomicBool,
    fut: RwLock<Option<JoinHandle<()>>>,

    result: Mutex<Vec<Result>>,
}

#[derive(Clone)]
pub struct ScanResults(Arc<ScanResultsImp>);

impl std::default::Default for ScanResults {
    fn default() -> Self {
        Self(Arc::new(Default::default()))
    }
}

impl ScanResults {
    pub fn get_instance() -> &'static Self {
        static INSTANCE: OnceCell<ScanResults> = once_cell::sync::OnceCell::new();

        INSTANCE.get_or_init(|| {
            Self::default()
        })
    }
}

impl ScanResults {
    pub fn run(&self) {
        let data: Vec<u8> = vec![0x1e, 0xff, 0xb0, 0x50, 0xa0, 0x00, 0x00, 0x03, 0xf5, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0x00, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5];

        let mut_fut = &mut *self.0.fut.write().unwrap();

        match mut_fut {
            None => {
                let this = self.clone();
                self.0.running.store(true, std::sync::atomic::Ordering::SeqCst);
                self.0.result.lock().unwrap().clear();

                let fut = 
                    async_std::task::spawn(async move {
                        loop {
                            if !this.0.running.load(std::sync::atomic::Ordering::SeqCst) {
                                break;
                            }

                            let _ = advertising::AdvertisingProcessor::get_instance().add_data(data.clone());

                            let _ = async_std::future::timeout(Duration::from_secs(2), future::pending::<()>());
                        }
                    });
                *mut_fut = Some(fut);
            }
            _ => {}
        }
    }

    pub async fn stop(&self) {
        let mut_fut = &mut *self.0.fut.write().unwrap();

        match mut_fut {
            None => {}
            Some(fut) => {
                self.0.running.store(false, std::sync::atomic::Ordering::SeqCst);
                fut.await;
                *mut_fut = None;
            }
        }
    }

    pub fn take(&self, cnt: usize) -> Vec<Result> {
        let mut_result = &mut *self.0.result.lock().unwrap();

        mut_result.drain(1..cnt).collect()
    }
}

unsafe impl Send for ScanResults {}
unsafe impl Sync for ScanResults {}

#[async_trait::async_trait]
impl ScanProcessorEventTrait for ScanResults {
    fn clone_as_event(&self) -> Box<dyn ScanProcessorEventTrait> {
        Box::new(self.clone())
    }

    async fn scan_event(&self, result: Vec<ScanResult>) {
        for r in result {
            self.0.result.lock().unwrap()
                .push(Result{
                    addr: r.addr,
                    data: r.data,
                })
        }
    }

}