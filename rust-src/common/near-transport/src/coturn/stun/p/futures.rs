
use std::{fmt::Display, future::Future, sync::{Arc, Mutex}, task::Waker, time::Duration };

use log::{error, info, trace, warn};

use near_base::{sequence::SequenceString, ErrorCode, NearError, NearResult};

use crate::{
    network::DataContext, 
    package::{AnyNamedRequest, PackageHeader, PackageHeaderExt}, 
    coturn::stun::p::provider::{BaseEventManager, BaseEventTrait}, 
    tunnel::{DynamicTunnel, PostMessageTrait}, 
    RequestorMeta, 
    Stack
};

use super::provider::AnyBaseEventCommand;

struct CallResult<T> {
    value: Option<NearResult<T>>,
    waker: Option<Waker>,
}

struct CallResultFuture<T> {
    result: Arc<Mutex<CallResult<T>>>
}

impl<T> Future for CallResultFuture<T> {
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

pub struct CallTemplate<T> {
    result: Arc<Mutex<CallResult<T>>>,
}

impl<T> CallTemplate<T>
where T: Send + Sync + Default + 'static + Display {

    pub async fn call(
        stack: Stack,
        tunnel: Option<DynamicTunnel>,
        requestor_meta: RequestorMeta,
        data: AnyNamedRequest,
        timeout: Option<Duration>,
    ) -> NearResult<(T, SequenceString)> {
        trace!(
            "CallTemplate::call: {data} package, tunnel-target:{:?}, requestor_meta: {requestor_meta}", 
            tunnel.as_ref().map(| tunnel | tunnel.peer_id())
        );

        let result = Arc::new(Mutex::new(CallResult{ value: None, waker: None }));
        let target = 
            if let Some(tunnel) = tunnel.as_ref() {
                Some(tunnel.peer_id())
            } else {
                requestor_meta.to.as_ref()
            }
            .ok_or_else(|| {
                warn!("missing target data");
                NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing target data")
            })?
            .clone();

        let sequence = 
            PostMessageTrait::post_message(
                &stack,
                (tunnel, requestor_meta, data, None, )
            )
            .await?;

        BaseEventManager::get_instance()
            .join_routine(
                &target,
                &sequence, 
                0, 
                Box::new(CallTemplate {
                    result: result.clone(),
                }) as Box<dyn BaseEventTrait>,
            )?;

        let fut = CallResultFuture::<T>{ result };

        let r = 
            if let Some(timeout) = timeout {
                async_std::future::timeout(timeout, fut)
                    .await
                    .map(| r | {
                        match r.as_ref() {
                            Ok(v) => {
                                info!("successfully get result: {v} at sequence: {sequence}");
                            }
                            Err(e) => {
                                error!("failed CallTemplate::call at sequence: {sequence} with err:{e}");
                            }
                        }
                        r
                    })
                    .map_err(| _ | {
                        error!("timeout CallTemplate::call at sequence: {sequence}");
                        NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, "timeout CallTemplate::call")
                    })?
            } else {
                fut.await
                    .map(| r | {
                        info!("successfully get result: {r} at sequence: {sequence}");
                        r
                    })
                    .map_err(| err | {
                        error!("failed CallTemplate::call at sequence: {sequence} with err:{err}");
                        err
                    })
            }?;

        Ok((r, sequence))
    }

}


#[async_trait::async_trait]
impl<T> BaseEventTrait for CallTemplate<T> 
where T: Send + Sync + Default + 'static {
    async fn emit(
        &self, 
        head: &PackageHeader,
        head_ext: &PackageHeaderExt,
        mut data: AnyBaseEventCommand,
    ) -> NearResult<()> {
        trace!(
            "CallTemplate<T>::emit: head: {}, head_ext: {}, begin...",
            head, head_ext,
        );

        let waker = {
            let result = &mut *self.result.lock().unwrap();
            let mut_data = AsMut::<T>::as_mut(&mut data);
            result.value = Some(Ok(std::mem::replace(mut_data, Default::default())));
            result.waker.take()
        };

        if let Some(w) = waker {
            w.wake();
        }

        Ok(())
    }

    async fn emit_error(
        &self, 
        error: NearError,
        data: DataContext,
    ) {
        let sequence = data.head.sequence();
        trace!(
            "CallTemplate<T>::emit_error: error: {error}, sequence: {sequence} begin...",
        );

        let waker = {
            let result = &mut *self.result.lock().unwrap();
            // let mut_error = &mut error;
            // result.value = Some(Ok(std::mem::replace(mut_data, Default::default())));
            result.value = Some(Err(error));
            result.waker.take()
        };

        if let Some(w) = waker {
            w.wake();
        }
    }

}


// #[async_trait::async_trait]
// impl<T> Routine<T, ()> for CallTemplate<T> 
// where T: Serialize + Deserialize + Send + Sync + std::default::Default + Clone {

//     async fn on_routine(&self, header_meta: &HeaderMeta, req: T) -> EventResult<()> {
//         trace!("RoutineTemplate::on_routine header_meta={header_meta}");

//         let r = match protos::RawObjectHelper::decode::<T>(req) {
//             Ok(data) => {
//                 match data {
//                     DataContent::Content(v) => DataContent::Content(v),
//                     DataContent::Error(e) => DataContent::Error(e),
//                 }
//             }
//             Err(e) => {
//                 let error_string = format!("failed decode message with err = {e}");
//                 error!("{error_string}, sequence = {}", header_meta.sequence());
//                 DataContent::Error(e)
//             }
//         };

//         let waker = {
//             let result = &mut *self.result.lock().unwrap();

//             match r {
//                 DataContent::Content(v) => result.value = Some(Ok(v)),
//                 DataContent::Error(e) => result.value = Some(Err(e)),
//             }
//             result.waker.take()
//         };

//         if let Some(w) = waker {
//             w.wake();
//         }

//         EventResult::Ignore
//     }
// }

