
use std::sync::{Mutex, Arc};

use async_std::channel::Sender;
use log::{trace, error};

use base::raw_object::RawObjectGuard;

use near_base::{Deserialize, Serialize};
use near_transport::{Routine, EventResult, HeaderMeta};
use protos::DataContent;

struct CbImpl<RESP> {
    snd: Sender<bool>,
    data: Mutex<DataContent<RESP>>,
}

#[derive(Clone)]
pub struct Cb<RESP>(Arc<CbImpl<RESP>>);

impl<RESP: std::default::Default> Cb<RESP> {
    pub fn new(snd: Sender<bool>) -> Box<Self> {
        Box::new(Self(Arc::new(CbImpl {
            snd,
            data: Mutex::new(DataContent::Content(RESP::default())),
        })))
    }

    pub fn take_result(&self) -> DataContent<RESP> {
        let mut result = DataContent::Content(RESP::default());
        let mut_data = &mut *self.0.data.lock().unwrap();
        std::mem::swap(mut_data, &mut result);

        result
    }
}

#[async_trait::async_trait]
impl<RESP> Routine<RawObjectGuard, RawObjectGuard> for Cb<RESP>
where RESP: Serialize + Deserialize + Send + Sync + std::default::Default + Clone + std::fmt::Display {
    async fn on_routine(&self, header_meta: &HeaderMeta, resp: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("Cb::on_routine, header_meta={}, resp={}", header_meta, &resp);

        let mut data = 
            match protos::RawObjectHelper::decode::<RESP>(resp) {
                Ok(resp) => resp,
                Err(e) => {
                    error!("failed decode message with err = {e}, sequence = {}", header_meta.sequence());
                    DataContent::Error(e)
                }
            };

        {
            let mut_data = &mut *self.0.data.lock().unwrap();
            std::mem::swap(mut_data, &mut data);
        }

        if let Err(e) = self.0.snd.send(true).await {
            let error_string = format!("failed send channel with err = {e}, sequence = {}", header_meta.sequence());
            error!("{error_string}");
        }

        EventResult::Ignore
    }
}

