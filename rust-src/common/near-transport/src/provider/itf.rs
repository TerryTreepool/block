
use near_base::{Serialize, Deserialize, };

use crate::tunnel::DynamicTunnel;

#[async_trait::async_trait]
pub trait ProcessEvent<REQ> {
    async fn on_event(req: REQ);
}

// pub trait ProcessTrait<T: Itf + Serialize + Deserialize> {
//     fn get_itf(command: u16) -> T;
// }

pub trait Itf: Send + Sync + 'static + std::fmt::Display {
}

pub struct ItfWrap {
    req: Box<dyn Itf>,
}

impl<T: Itf + Serialize + Deserialize> From<T> for ItfWrap {
    fn from(r: T) -> Self {
        Self {
            req: Box::new(r),
        }
    }
}

impl std::ops::Deref for ItfWrap {
    type Target = dyn Itf;

    fn deref(&self) -> &Self::Target {
        self.req.as_ref()
    }
}

// pub struct Itf<REQ, RESP>
// where
//     REQ:  Send + Sync + 'static + Serialize + Deserialize + std::fmt::Display,
//     RESP: Send + Sync + 'static + Serialize + Deserialize + std::fmt::Display,
// {
//     pub request:  REQ,
//     pub response: Option<NearResult<RESP>>,
// }
