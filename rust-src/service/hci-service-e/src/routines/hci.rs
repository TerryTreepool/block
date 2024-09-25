
use enumflags2::make_bitflags;
use log::error;
use near_base::{NearResult, NearError, ErrorCode};

pub struct HciSocket {
    stream: bluez::management::ManagementStream,
}

unsafe impl Send for HciSocket {}
unsafe impl Sync for HciSocket {}

impl HciSocket {
    pub fn open() -> NearResult<Self> {
        let stream = 
            bluez::management::ManagementStream::open()
                .map_err(| err | {
                    let error_string = format!("failed open hci-socket with err = {err}");
                    error!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_3RD, error_string)
                })?;

        Ok(Self{
            stream,
        })
    }
}

impl HciSocket {
    pub async fn adverting(&mut self, adv_data: Vec<u8>) {
        use bluez::management::AdvertisingFlags;
/*
        // let f = make_bitflags!({EnterConnectable | AdvertiseDiscoverable});
        let params = bluez::management::AdvertisingParams {
            instance: 10,
            flags: make_bitflags!(AdvertisingFlags::{EnterConnectable | AdvertiseDiscoverable}),
            duration: 2,
            timeout: 0,
            adv_data,
            scan_rsp: vec![],
        };

        let (snd, mut rcv) = async_std::channel::bounded(std::mem::size_of::<bluez::management::Response>());

        async_std::task::spawn(async move {
            use bluez::management::Response;

            let r: Response = match rcv.recv().await {
                Ok(r) => {
                    r
                }
                Err(e) => {
                    error!("failed recv with err = {e}");
                    return;
                }
            };

            println!("event: {:?}, controller: {}", r.event, r.controller);
        });

        match bluez::management::add_advertising(&mut self.stream, 
                                                 bluez::management::Controller::new(0),
                                                 params, 
                                                 Some(snd))
                .await {
            Ok(_) => {},
            Err(e) => {
                error!("failed advertising with err = {e}");
            }
        }
*/
    }
}
