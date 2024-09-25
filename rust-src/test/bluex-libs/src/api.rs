
use std::{sync::RwLock, time::Duration};

use near_base::*;

use crate::{hci::{advertising::AdvertisingProcessor, scanning::{ScanProcessor, ScanProcessorEventTrait}}, ScanResults};

lazy_static::lazy_static!(
    pub static ref LAST_ERROR: RwLock<NearError> = RwLock::new(NearError::default());
);

fn set_last_error(error: NearError) {
    let mut_error = &mut *LAST_ERROR.write().unwrap();
    *mut_error = error;
    //  *(&mut LAST_ERROR.write.unwrap()) = error;
}

fn get_last_error() -> NearError {
    std::mem::replace(&mut *LAST_ERROR.write().unwrap(), Default::default())
}

pub fn blue_adverting() -> u16 {

    async_std::task::block_on(async move {
        match AdvertisingProcessor::get_instance()
                .active()
                .await {
            Ok(()) => {},
            Err(err) => {
                let errno = err.errno();
                match errno {
                    ErrorCode::NEAR_ERROR_STARTUP => {},
                    _ => {
                        set_last_error(err);
                        return errno.into_u16();
                    }
                }
            }
        };

        match ScanProcessor::get_instance()
                .active(Duration::ZERO, ScanResults::get_instance().clone_as_event())
                .await {
            Ok(()) => {},
            Err(err) => {
                let errno = err.errno();
                match errno {
                    ErrorCode::NEAR_ERROR_STARTUP => {},
                    _ => {
                        set_last_error(err);
                        return errno.into_u16();
                    }
                }
            }
        }

        ScanResults::get_instance().run();

        ErrorCode::NEAR_ERROR_SUCCESS.into_u16()
    })

}

pub fn blue_stop() {
    async_std::task::block_on(async move {
        let _ = AdvertisingProcessor::get_instance().wait_and_close().await;
        let _ = ScanProcessor::get_instance().wait_and_close().await;

        ScanResults::get_instance().stop().await;
    });
}

pub struct Result {
    pub addr: [u8; 6],
    pub data: Vec<u8>,
}

pub fn blue_take(cnt: usize) -> Vec<Result> {
    ScanResults::get_instance().take(cnt)
}