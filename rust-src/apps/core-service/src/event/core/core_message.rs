
use std::sync::Arc;

use log::{error, warn};

use near_base::{NearResult, NearError, ErrorCode, ObjectId,
    };
use near_transport::{ProcessTrait, RoutineEventTrait, RoutineWrap, EventResult, Routine, HeaderMeta,
    };
use near_util::TopicRef;

use base::raw_object::RawObjectGuard;
use topic_util::topic_types::{TOPIC_P_CORE_LABEL, TOPIC_S_SUBSCRIBE_LABEL, TOPIC_S_DISSUBSCRIBE_LABEL};
use protos::{core_message::{Subscribe_message, Dissubscribe_message, }, 
             DataContent, RawObjectHelper
    };

use crate::event::MessageTrait;

use super::CoreMessageCallbackTrait;

struct CoreMessageImpl {
    cb: Box<dyn CoreMessageCallbackTrait>,
}

#[derive(Clone)]
pub struct CoreMessage(Arc<CoreMessageImpl>);

impl CoreMessage {
    pub(crate) fn new(cb: Box<dyn CoreMessageCallbackTrait>) -> Self {
        Self(Arc::new(CoreMessageImpl{
            cb,
        }))
    }

}

impl MessageTrait for CoreMessage {
    fn clone_as_message(&self) -> Box<dyn MessageTrait> {
        Box::new(self.clone())
    }

    fn is_core_message(&self) -> bool {
        true
    }

    fn primary_label(&self) -> Option<&str> {
        Some(TOPIC_P_CORE_LABEL)
    }
}

impl ProcessTrait for CoreMessage {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, _from: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        if let Some(secondary) = *topic.secondary() {
            if secondary.eq_ignore_ascii_case(TOPIC_S_SUBSCRIBE_LABEL) {
                Ok(RoutineWrap::new(OnSubscribeMessageRoutine::new(self.0.cb.clone_as_coremessage())) as Box<dyn RoutineEventTrait>) 
            } else if secondary.eq_ignore_ascii_case(TOPIC_S_DISSUBSCRIBE_LABEL) {
                Ok(RoutineWrap::new(OnDissubscribeMessageRoutine::new(self.0.cb.clone_as_coremessage())) as Box<dyn RoutineEventTrait>) 
            } else {
                Err(NearError::new(ErrorCode::NEAR_ERROR_TOPIC_SECONDARY, format!("The [{}] secondary topic cann't found.", topic)))
            }
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_TOPIC_SECONDARY, format!("The [{}] secondary loss.", topic)))
        }
    }
}

/// OnSubscribeMessageRoutine
pub struct OnSubscribeMessageRoutine {
    cb: Box<dyn CoreMessageCallbackTrait>,
}

impl OnSubscribeMessageRoutine {
    pub fn new(cb: Box<dyn CoreMessageCallbackTrait>,) -> Box<Self> {
        Box::new(Self{
            cb
        })
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for OnSubscribeMessageRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {

        let subscribe_message = 
            match RawObjectHelper::decode::<Subscribe_message>(req) {
                Ok(message) => {
                    if let DataContent::Content(m) = message {
                        m
                    } else {
                        error!("The subscribe message is error.");
                        return EventResult::Ignore;
                    }
                }
                Err(e) => {
                    error!("failed decode pb-message with err = {e}");
                    return EventResult::Ignore;
                }
            };

        for mut m in subscribe_message.messge {
            let _ = self.cb
                        .on_subscribe(&header_meta.requestor, m.take_message().into(), (m.mt() as i32).try_into().unwrap_or_default())
                        .map_err(| e | {
                            warn!("Warning: can't subscribe message for {}", header_meta.requestor);
                            e
                        });
        }

        if let Ok(r) = RawObjectHelper::encode_none() {
            EventResult::Response(r.into())
        } else {
            EventResult::Ignore
        }
    }

}

/// OnDissubscribeMessageRoutine
pub struct OnDissubscribeMessageRoutine {
    cb: Box<dyn CoreMessageCallbackTrait>,
}

impl OnDissubscribeMessageRoutine {
    pub fn new(cb: Box<dyn CoreMessageCallbackTrait>) -> Box<Self> {
        Box::new(Self{cb})
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for OnDissubscribeMessageRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {

        let message = match RawObjectHelper::decode::<Dissubscribe_message>(req) {
            Ok(message) => {
                if let DataContent::Content(m) = message {
                    m
                } else {
                    error!("The dis-subscribe message is error.");
                    return EventResult::Ignore;
                }
            }
            Err(e) => {
                error!("failed decode pb-message with err = {e}");
                return EventResult::Ignore;
            }
        };

        let _ = 
            self.cb
                .on_dissubscribe(&header_meta.requestor, message.message_name.into())
                .map_err(| err | {
                    warn!("Warning: can't subscribe message for {}", header_meta.requestor);
                    err
                });

        
        if let Ok(r) = RawObjectHelper::encode_none() {
            EventResult::Response(r.into())
        } else {
            EventResult::Ignore
        }
        
    }

}
