
use std::{sync::{RwLock, Arc}, collections::BTreeMap, pin::Pin};

use log::{info, error};

use near_base::{NearResult, NearError, ErrorCode};
use near_transport::RoutineEventTrait;
use near_util::{TopicRef, Topic};

use base::{MessageExpire, MessageType};

pub trait TopicRoutineCbEventTrait: Send + Sync  {
    fn on_topic_routine(&self) -> NearResult<Box<dyn RoutineEventTrait>>;
}

impl<F> TopicRoutineCbEventTrait for F
where
    F: Send + Sync + 'static + Fn() -> NearResult<Box<dyn RoutineEventTrait>>
{
    fn on_topic_routine(&self) -> NearResult<Box<dyn RoutineEventTrait>> {
        let fut = (self)();
        let res = fut?;
        Ok(res)
    }
}

pub trait TopicRoutineOpEventTrait: Send + Sync {
    fn subscribe_message(&self, topic: &Topic, expire: MessageExpire, mt: Option<MessageType>) -> NearResult<()>;
    fn dissubscribe_message(&self, topic: &Topic) -> NearResult<()>;
}

struct TopicValue {
    mt: Option<MessageType>,
    routine_cb_event: Arc<dyn TopicRoutineCbEventTrait>,
}

struct ManagerImpl {
    event_callback: Pin<Box<dyn TopicRoutineOpEventTrait>>,
    topics: RwLock<BTreeMap<Topic, TopicValue>>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub(crate) fn new(event_callback: Pin<Box<dyn TopicRoutineOpEventTrait>>) -> Self {
        Self(Arc::new(ManagerImpl{
            event_callback,
            topics: RwLock::new(BTreeMap::new()),
        }))
    }
}

impl Manager {

    pub fn register_public_topic(
        &self,
        topic: &Topic, 
        topic_event: impl TopicRoutineCbEventTrait + 'static,
    ) -> NearResult<()> {
        self.register_topic_event(topic, topic_event, Some(MessageType::Public))
    }

    pub fn register_private_topic(
        &self,
        topic: &Topic, 
        topic_event: impl TopicRoutineCbEventTrait + 'static,
    ) -> NearResult<()> {
        self.register_topic_event(topic, topic_event, Some(MessageType::Private))
    }

    pub fn reregister_topic_event(
        &self
    ) {
        let topics: Vec<(Topic, Option<MessageType>)> = {
            self.0.topics.read().unwrap()
                .iter()
                .map(| (k, v) | {
                    (k.clone(), v.mt)
                })
                .collect()
        };

        for (topic, mt) in topics {
            let _ = 
                self.0
                    .event_callback
                    .subscribe_message(&topic, Default::default(), mt)
                    .map(|_| {
                        info!("success suscribe {} message.", topic);
                    })
                    .map_err(| err | {
                        let error_string = format!("failed post subscribe message with err = {}", err);
                        error!("{error_string}");
                        err
                    });
        }

    }

    pub(in self) fn register_topic_event(
        &self,
        topic: &Topic, 
        topic_event: impl TopicRoutineCbEventTrait + 'static,
        mt: Option<MessageType>,
    ) -> NearResult<()> {
        let newly = {
            let topics = &mut *self.0.topics.write().unwrap();
            match topics.get(topic) {
                Some(_event) => false,
                None => {
                    let dyn_topic_routine = Arc::new(topic_event);
                    topics.insert(topic.clone(), TopicValue { mt, routine_cb_event: dyn_topic_routine });
                    true
                }
            }
        };

        if newly {
            self.0
                .event_callback
                .subscribe_message(topic, Default::default(), mt)
                .map(|_| {
                    info!("success suscribe {} message.", topic);
                })
                .map_err(| err | {
                    let error_string = format!("failed post subscribe message with err = {}", err);
                    error!("{error_string}");
                    err
                })
        } else {
            Ok(())
        }
    }

    pub fn call(&self, topic: &TopicRef<'_>) -> NearResult<Box<dyn RoutineEventTrait>> {
        let routine =
            match self.0.topics.read().unwrap()
                      .get(topic.topic()) {
            Some(routine) => Ok(routine.routine_cb_event.clone()),
            None => Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Cloud not found the [{}] topic, it maybe not registed.", topic))),
        }?;

        routine.on_topic_routine()
    }

}
