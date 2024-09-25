
use std::{task::Waker, sync::{Arc, Mutex}, future::Future};

use log::{error, trace};

use near_base::{NearResult, Serialize, Deserialize, NearError};
use near_transport::{HeaderMeta, EventResult, Routine, RoutineWrap, RequestorMeta};
use near_util::Topic;

use base::raw_object::RawObjectGuard;
use protos::DataContent;

use crate::RuntimeStack;

struct RoutineResult<T> {
    value: Option<NearResult<T>>,
    waker: Option<Waker>,
}

struct RoutineResultFuture<T> {
    result: Arc<Mutex<RoutineResult<T>>>
}

impl<T> Future for RoutineResultFuture<T> {
    type Output = NearResult<T>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let result = &mut *self.result.lock().unwrap();
        match result.value.as_mut() {
            Some(r) => {
                let mut ret = Err(NearError::default());
                std::mem::swap(r, &mut ret);
                std::task::Poll::Ready(ret)
            }
            None => {
                result.waker = Some(cx.waker().clone());
                std::task::Poll::Pending
            }
        }
    }
}

pub struct RoutineTemplate<T> {
    result: Arc<Mutex<RoutineResult<T>>>,
}

impl<T> RoutineTemplate<T> 
where T: Serialize + Deserialize + Send + Sync + std::default::Default + Clone + 'static {

    pub async fn call<R: Serialize>(
        topic: Topic, 
        raw_data: R
    ) -> NearResult<impl Future<Output = NearResult<T>>> {
        trace!("RoutineTemplate::call: topic: {}", topic);

        let raw_data_req = protos::RawObjectHelper::encode_with_raw(raw_data)?;

        let result = Arc::new(Mutex::new(RoutineResult::<T>{ value: None, waker: None }));

        RuntimeStack::get_instance()
            .stack()
            .post_message(
                RequestorMeta {
                    topic: Some(topic),
                    ..Default::default()
                },
                raw_data_req, 
                Some(RoutineWrap::new(Box::new(RoutineTemplate{
                    result: result.clone(),
                })))
            )
            .await?;

        Ok(RoutineResultFuture {
            result
        })
    }

    pub async fn call_with_headermeta<R: Serialize>(
        header_meta: &HeaderMeta, 
        topic: Topic, 
        raw_data: R
    ) -> NearResult<impl Future<Output = NearResult<T>>> {
        trace!("RoutineTemplate::call_with_headermeta: header_meta: {}, topic: {}", header_meta, topic);
        let raw_data_req = protos::RawObjectHelper::encode_with_raw(raw_data)?;

        let result = Arc::new(Mutex::new(RoutineResult::<T>{ value: None, waker: None }));

        RuntimeStack::get_instance()
            .stack()
            .post_message(
                RequestorMeta {
                    sequence: Some(header_meta.sequence().clone()),
                    creator: header_meta.creator.clone(),
                    topic: Some(topic),
                    ..Default::default()
                },
                raw_data_req, 
                Some(RoutineWrap::new(Box::new(RoutineTemplate{
                    result: result.clone(),
                })))
            )
            .await?;

        Ok(RoutineResultFuture {
            result
        })
    }

}

#[async_trait::async_trait]
impl<T> Routine<RawObjectGuard, RawObjectGuard> for RoutineTemplate<T> 
where T: Serialize + Deserialize + Send + Sync + std::default::Default + Clone {

    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("RoutineTemplate::on_routine header_meta={header_meta}");

        let r = match protos::RawObjectHelper::decode::<T>(req) {
            Ok(data) => {
                match data {
                    DataContent::Content(v) => DataContent::Content(v),
                    DataContent::Error(e) => DataContent::Error(e),
                }
            }
            Err(e) => {
                let error_string = format!("failed decode message with err = {e}");
                error!("{error_string}, sequence = {}", header_meta.sequence());
                DataContent::Error(e)
            }
        };

        let waker = {
            let result = &mut *self.result.lock().unwrap();

            match r {
                DataContent::Content(v) => result.value = Some(Ok(v)),
                DataContent::Error(e) => result.value = Some(Err(e)),
            }
            result.waker.take()
        };

        if let Some(w) = waker {
            w.wake();
        }

        EventResult::Ignore
    }
}

