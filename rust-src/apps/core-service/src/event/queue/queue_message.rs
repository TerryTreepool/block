
use std::{sync::{Arc, RwLock}, collections::{BTreeMap, btree_map::Entry}};

use base::MessageType;
use log::warn;
use near_base::{ObjectId, NearResult, ErrorCode, NearError};
use near_transport::{ProcessTrait, RoutineEventTrait};
use near_util::{Topic, TopicRef};

use crate::event::MessageTrait;

use super::{sub_message::SubMessage, DispatchCallbackTrait, };

struct MessageQueueImpl {
    // TODO: tire is better than btree
    cb: Box<dyn DispatchCallbackTrait>,
    sub_messages: RwLock<BTreeMap<String, SubMessage>>,
}

#[derive(Clone)]
pub struct MessageQueue(Arc<MessageQueueImpl>);

impl MessageQueue {
    pub fn new(cb: Box<dyn DispatchCallbackTrait>) -> Self {
        Self(Arc::new(MessageQueueImpl{
            cb,
            sub_messages: RwLock::new(BTreeMap::new()),
        }))
    }

    pub(self) fn sub_message_of(&self, primary_topic: &str) -> Option<SubMessage> {
        self.0.sub_messages.read().unwrap()
            .get(primary_topic)
            .cloned()
    }

    pub(self) fn create_sub_message(&self, primary_topic: &str) -> SubMessage {
        match self.sub_message_of(primary_topic) {
            Some(sub_message) => sub_message, 
            None => {
                match self.0
                          .sub_messages
                          .write().unwrap()
                          .entry(primary_topic.to_owned()) {
                    Entry::Vacant(empty) => {
                        let sub_message = SubMessage::new(self.0.cb.clone_as_dispatch());
                        empty.insert(sub_message.clone());
                        sub_message
                    }
                    Entry::Occupied(exist) => {
                        exist.get().clone()
                    }
                }
            }
        }
    }

    pub fn subscribe(&self, from: &ObjectId, topic: Topic, mt: MessageType) -> NearResult<()> {
        let topic_ref = topic.topic_d()?;

        self.create_sub_message(topic_ref.primary())
            .subscribe(from, topic, mt)
    }

    pub fn dissubscribe(&self, from: &ObjectId, topic: Topic) -> NearResult<()> {
        let topic_ref = topic.topic_d()?;

        self.sub_message_of(topic_ref.primary())
            .ok_or_else(|| {
                let error_string = format!("Cloud not found the primary [{}] topic label.", topic_ref.primary());
                warn!("{error_string}");
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
            })?
            .dissubscribe(from, topic)
    }
}

impl MessageTrait for MessageQueue {
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

impl ProcessTrait for MessageQueue {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, sender: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        self.sub_message_of(topic.primary())
            .ok_or_else(|| {
                let error_string = format!("Cloud not found the primary [{}] topic label.", topic.primary());
                // warn!("{error_string}");
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
            })?
            .create_routine(sender, topic)
    }
}
