
use std::{sync::{Arc, RwLock}, collections::BTreeMap};

use near_base::{ObjectId, NearResult, ErrorCode, NearError};
use near_transport::{ProcessTrait, RoutineEventTrait};
use near_util::{Topic, TopicRef};

use crate::event::MessageTrait;

use super::{sub_message::{SubMessage, }, DispatchCallbackTrait, DispatchMessage};

struct MessageImpl {
    // TODO: tire is better than btree
    cb: Box<dyn DispatchCallbackTrait<DispatchMessage>>,
    sub_messages: RwLock<BTreeMap<String, SubMessage>>,
}

#[derive(Clone)]
pub struct Message(Arc<MessageImpl>);

impl Message {
    pub fn new(cb: Box<dyn DispatchCallbackTrait<DispatchMessage>>) -> Self {
        Self(Arc::new(MessageImpl{
            cb,
            sub_messages: RwLock::new(BTreeMap::new()),
        }))
    }

    pub fn subscribe(&self, from: &ObjectId, topic: Topic) -> NearResult<()> {
        let topic_ref = topic.topic_d()?;

        let sub_message = {
            let sub_messages = &mut *self.0.sub_messages.write().unwrap();

            match sub_messages.get(topic_ref.primary()) {
                Some(sub_message) => sub_message.clone(),
                None => {
                    let sub_message = SubMessage::new(self.0.cb.clone_as_dispatch());
                    let _ = sub_messages.insert(topic_ref.primary().to_string(), sub_message.clone());
                    sub_message
                }
            }
        };

        sub_message.subscribe(from, topic_ref.secondary().unwrap_or(""))
    }

    pub fn dissubscribe(&self, from: &ObjectId, topic: Topic) -> NearResult<()> {
        let topic_ref = topic.topic_d()?;

        let sub_message = {
            self.0.sub_messages.read().unwrap()
                .get(topic_ref.primary())
                .cloned()
        }
        .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Cloud not found the primary [{}] topic label.", topic_ref.primary())) )?;

        sub_message.dissubscribe(from, topic_ref.secondary().unwrap_or(""))
    }
}

impl MessageTrait for Message {
    fn clone_as_message(&self) -> Box<dyn MessageTrait> {
        Box::new(self.clone())
    }

    fn is_core_message(&self) -> bool {
        false
    }

    fn primary_label(&self) -> Option<&str> {
        None
    }

}

impl ProcessTrait for Message {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, sender: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        let message = 
            self.0.sub_messages.read().unwrap()
                .get(topic.primary())
                .map(| message | message.clone())
                .ok_or(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Cloud not {} message", topic.primary())))?;

        message.create_routine(sender, topic)

    }
}
