
use std::{sync::{RwLock, Arc}, collections::{BTreeMap, }};

use near_base::{NearResult, NearError, ErrorCode};
use near_transport::{RoutineEventTrait, topic::{TopicRef, Topic}};

pub trait TopicRoutineTrait: Send + Sync  {
    fn on_topic_routine(&self) -> NearResult<Box<dyn RoutineEventTrait>>;
}

struct ManagerImpl {
    topics: RwLock<BTreeMap<Topic, Box<dyn TopicRoutineTrait>>>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl std::default::Default for Manager {
    fn default() -> Self {
        Self(Arc::new(ManagerImpl{
            topics: RwLock::new(BTreeMap::new()),
        }))
    }
}

impl Manager {

    pub fn register_topic_event(&self, topic: Topic, topic_event: Box<dyn TopicRoutineTrait>) {
        self.0.topics.write().unwrap()
            .entry(topic)
            .or_insert(topic_event);
    }

    pub fn call(&self, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        match self.0.topics.read().unwrap()
                    .get(topic.topic()) {
            Some(routine) => routine.on_topic_routine(),
            None => Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Cloud not found the [{}] topic, it maybe not registed.", topic))),
        }
    }
}
