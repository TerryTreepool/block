
use std::sync::Arc;

use base::{SubscribeMessage, DissubcribeMessage, TOPIC_P_CORE_LABEL, TOPIC_S_SUBSCRIBE_LABEL, TOPIC_S_DISSUBSCRIBE_LABEL, };
use near_base::{NearResult, NearError, ErrorCode, ObjectId,
    };
use near_transport::{ProcessTrait, RoutineEventTrait, RoutineWrap, EventResult, Routine, process::EmptyTrait,
    };
use near_util::{TopicRef, Topic};

use crate::event::{MessageTrait, };

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
impl Routine<SubscribeMessage, EmptyTrait> for OnSubscribeMessageRoutine {
    async fn on_routine(&self, from: &ObjectId, req: SubscribeMessage) -> EventResult<EmptyTrait> {

        for (message, _expire) in req.message_list {
            let _ = self.cb.on_subscribe(from, Topic::from(message));
        }

        EventResult::Ingnore

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
impl Routine<DissubcribeMessage, EmptyTrait> for OnDissubscribeMessageRoutine {
    async fn on_routine(&self, from: &ObjectId, req: DissubcribeMessage) -> EventResult<EmptyTrait> {

        let _ = self.cb
                    .on_dissubscribe(from, Topic::from(req.message));

        EventResult::Ingnore

    }

}
