
use std::{sync::{RwLock, Arc}, collections::{BTreeMap, }};

use log::info;
use near_base::{NearResult, NearError, ErrorCode};
use near_core::near_error;
use near_transport::{RoutineEventTrait, topic::{TopicRef, Topic, }};

use crate::NdsStack;

pub trait TopicRoutineTrait: Send + Sync  {
    fn on_topic_routine(&self) -> NearResult<Box<dyn RoutineEventTrait>>;
}

struct ManagerImpl {
    nds_stack: NdsStack,
    topics: RwLock<BTreeMap<Topic, Arc<dyn TopicRoutineTrait>>>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn new(nds_stack: NdsStack) -> Self {
        Self(Arc::new(ManagerImpl{
            nds_stack,
            topics: RwLock::new(BTreeMap::new()),
        }))
    }
}

impl Manager {

    pub fn register_topic_event(&self, 
                                topic: &TopicRef, topic_event: Arc<dyn TopicRoutineTrait>) {
        let newly = {
            let topics = &mut *self.0.topics.write().unwrap();
            match topics.get(topic.topic()) {
                Some(_event) => false,
                None => {
                    topics.insert(topic.topic().clone(), topic_event);
                    true
                }
            }
        };

        if newly {
            let _ = 
            self.0
                .nds_stack
                .subscribe_message(topic.topic().clone().into(), base::MessageExpire::Normal)
                .map(|_| {
                    info!("success suscribe {} message.", topic);
                })
                .map_err(| err | {
                    near_error!(err.errno(), format!("failed post subscribe message with err = {}", err));
                });
        }
    }

    pub(crate) fn call(&self, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        let routine = 
            match self.0.topics.read().unwrap()
                      .get(topic.topic()) {
            Some(routine) => Ok(routine.clone()),
            None => Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Cloud not found the [{}] topic, it maybe not registed.", topic))),
        }?;
        
        routine.on_topic_routine()
    }
}
