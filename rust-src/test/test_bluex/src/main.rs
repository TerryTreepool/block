
use async_std::future;
use bluex::{Config, management::{stream::StreamFlags, scaner::{ScanParameter, ScanSwitch, Scanning, ScanResult}}};
use enumflags2::make_bitflags;
use log::debug;
use near_core::{LoggerBuilder, get_log_path};

async fn scanning() {
    async_std::task::block_on(async move {
        let mut bluex = 
        bluex::management::stream::Stream::open_default(make_bitflags!(StreamFlags::{NonBlock})).expect("failed");

        ScanParameter::default()
            .cmd(&bluex)
            .expect("failed");

        ScanSwitch::open_scan()
            .cmd(&bluex)
            .expect("failed");

        let (snd, rcv) = async_std::channel::bounded::<ScanResult>(10);

        async_std::task::spawn(async move {
            loop {
                if let Ok(r) = rcv.recv().await {
                    debug!(r#"successful recv scan result, addr={}, data = {}"#, r.addr, hex::encode_upper(r.data));
                } else {
                    break;
                }
            }
        });

        let mut scanning = Scanning::open(bluex).expect("failed open hci for scanning.");

        loop {
            if let Ok(r) = scanning.scanning().await {
                debug!(r#"
                {}
                {}"#, r.addr.to_string(), hex::encode(r.data));
            } else {
                let _ = async_std::future::timeout(std::time::Duration::from_millis(100), async_std::future::pending::<()>()).await;
            }
        }
    });
}

#[async_std::main]
async fn main() {
    LoggerBuilder::new("test_bluex", get_log_path().join("test_bluex"))
    .set_level(near_core::LogLevel::Trace)
    .set_console(near_core::LogLevel::Trace)
    .build()
    .expect("failed");

    let bluex = bluex::management::stream::Stream::open_default(Default::default()).expect("failed");

    bluex::management::adverting::AdvertisingParam
        ::new(1000, 1000)
        .cmd(&bluex)
        .expect("failed");

    bluex::management::adverting::AdvertisingSwitch
        ::open_advertising()
        .cmd(&bluex)
        .expect("failed");

    bluex::management::adverting::AdvertisingData
        ::new(vec![0x1e, 0xff, 0x54, 0xb1, 0xa0, 0x01, 0xff, 0x03, 0xf5, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0x00, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5, 0xf5])
        .unwrap()
        .cmd(&bluex)
        .expect("faied");

    future::timeout(std::time::Duration::from_secs(6000), scanning()).await;

    bluex::management::adverting::AdvertisingSwitch
        ::close_advertising()
        .cmd(&bluex)
        .expect("failed");


}
