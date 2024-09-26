
use std::{sync::{atomic::AtomicBool, Arc}, time::Duration, vec};
use async_std::{task::JoinHandle, sync::RwLock};


use bluex::management::{stream::{Stream, StreamFlags}, scaner::{Scanning, ScanResult}};
use enumflags2::make_bitflags;

use log::{debug, error, info, trace};
use near_base::{Timestamp, NearResult, now};

use once_cell::sync::OnceCell;

const CHANNEL_CAPCITY_MAX: usize = 30;

struct ScanActive {
    begin_scanning: Timestamp,
    timeout_us: Duration,
    activing: AtomicBool,
    fut: Vec<JoinHandle<()>>,
}

enum ScanStatus {
    UnActive,
    Actived(Arc<ScanActive>),
}

#[async_trait::async_trait]
pub trait ScanProcessorEventTrait: Send + Sync {
    fn clone_as_event(&self) -> Box<dyn ScanProcessorEventTrait>;
    async fn scan_event(&self, result: Vec<ScanResult>);
}

pub struct ScanProcessor {
    status: RwLock<ScanStatus>,
}

impl ScanProcessor {
    pub fn get_instance() -> &'static ScanProcessor {
        static INSTANCE: OnceCell<ScanProcessor> = OnceCell::new();

        INSTANCE.get_or_init(|| {
            let c = ScanProcessor {
                status: RwLock::new(ScanStatus::UnActive),
            };
            c
        })
    }

    pub async fn is_actived(&self) -> bool {
        match &*self.status.read().await {
            ScanStatus::Actived(_) => true,
            _ => false,
        }
    }

    pub async fn active(&self, timeout_us: Duration, event: Box<dyn ScanProcessorEventTrait>) -> NearResult<()> {
        let w = &mut *self.status.write().await;
        match w {
            ScanStatus::Actived(_) => Ok(()),
            ScanStatus::UnActive => {
                let status = Arc::new(ScanActive {
                    begin_scanning: now(),
                    timeout_us,
                    activing: AtomicBool::new(true),                
                    fut: {
                        let fut = async_std::task::spawn(async move {
                            let _ = ScanProcessor::get_instance().process(event).await;
                        });

                        vec![fut]
                    },
                });

                *w = ScanStatus::Actived(status);

                Ok(())
            }
        }    
    }

    pub async fn wait_and_close(&self) {
        trace!("wait_and_close");

        if !self.is_actived().await {
            debug!("Scanning module has been unactived.");
            return;
        } else {
            let w = &mut *self.status.write().await;
            match w {
                ScanStatus::Actived(status) => {
                    // set disactive
                    status.activing.store(false, std::sync::atomic::Ordering::SeqCst);

                    let mut fut = vec![];
                    {
                        std::mem::swap(&mut fut, &mut unsafe {&mut *(Arc::as_ptr(status) as *mut ScanActive) }.fut);
                    }

                    let _ = futures::future::join_all(fut).await;

                    *w = ScanStatus::UnActive;
                }
                ScanStatus::UnActive => {}
            }
        }
    }
}

impl ScanProcessor {

    async fn process(&self, event: Box<dyn ScanProcessorEventTrait>) {
        let status = 
            if let ScanStatus::Actived(status) = &*self.status.read().await {
                status.clone()
            } else {
                unreachable!();
            };

        debug!("open hci-socket stream for scan.");
        // stream
        let stream = match Stream::open_default(make_bitflags!(StreamFlags::{NonBlock})) {
            Ok(stream) => stream,
            Err(e) => {
                error!("failed open hci with err: {e}");
                return;
            }
        };

        debug!("set scan-parameter.");
        // set scan parameter
        if let Err(e) = bluex::management::scaner::ScanParameter::default().cmd(&stream) {
            error!("failed set-scan-paramter with err: {e}");
            return;
        }

        debug!("open scan-switch.");
        // enable scan
        if let Err(e) = bluex::management::scaner::ScanSwitch::open_scan().cmd(&stream) {
            error!("failed open-scan with err: {e}");
            return;
        }

        let (snd, rcv) = async_std::channel::bounded::<ScanResult>(CHANNEL_CAPCITY_MAX);

        // start scanning
        let mut scanning = match Scanning::open(stream.clone()) {
            Ok(scanning) => scanning,
            Err(e) => {
                error!("failed init Scanning with err: {e}");
                return;
            }
        };

        // start scan
        async_std::task::spawn(async move {
            let begin_ms = now();
            let timeout_ms = std::time::Duration::from_millis(100);
            let report_ms = std::time::Duration::from_secs(2).as_micros() as u64;
            let mut array = vec![];

            loop {
                let rcv_clone = rcv.clone();

                let r = match async_std::future::timeout(timeout_ms, rcv_clone.recv()).await {
                    Ok(r) => {
                        r.map_or_else(| e | {
                            error!("failed to recv from channel with err : {e}");
                            false
                        }, | data | {
                            array.push(data);
                            true
                        })
                    }
                    Err(_e) => {
                        true
                    }
                };

                let need_report = {

                    if now() - begin_ms > report_ms { // timeout 
                        true
                    } else if array.len() >= CHANNEL_CAPCITY_MAX / 2 {  // array is full
                        true
                    } else {    // exit
                        !r
                    }

                };

                if need_report {
                    let mut mut_data = vec![];
                    std::mem::swap(&mut mut_data, &mut array);
                    let event_clone = event.clone_as_event();

                    async_std::task::spawn(async move {
                        event_clone.scan_event(mut_data).await;
                    });
                }

                if !r {
                    break;
                }                

            }
        });

        let timeout_us = status.timeout_us.as_micros() as u64;
        
        loop {
            if !status.activing.load(std::sync::atomic::Ordering::SeqCst) {
                info!("Preparing to end the scan-hci process.");
                break;
            }

            if timeout_us > 0 &&
               now() - status.begin_scanning > timeout_us {
                info!("time expired, ready to end scan-hci.");
                break;
            }

            if let Ok(data) = scanning.scanning().await {
                let _ = snd.send(data).await;
            } else {
                let _ = async_std::future::timeout(Duration::from_millis(100), async_std::future::pending::<()>()).await;
            }
        }

        debug!("close scan-switch.");
        // close scanning
        let _ = bluex::management::scaner::ScanSwitch::close_scan().cmd(&stream);

        // end scanning
    }

}
