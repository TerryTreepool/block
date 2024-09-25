
use std::{sync::{Arc, RwLock, }, 
          collections::{BTreeMap, BTreeSet}, 
    };

use log::{info, };
use near_base::{NearResult, ObjectId, DynamicPtr};
use near_transport::{ProcessTrait, RoutineEventTrait, topic::TopicRef};

use super::{Manager, MessageTrait, };

struct MessageCenterImpl {
    manager: Manager,
    subscribe_message: RwLock<BTreeMap<String, ObjectIdsPtr>>,
}

type ObjectIdPtr = DynamicPtr<ObjectId>;

struct ObjectIds {
    ids: RwLock<BTreeSet<DynamicPtr<ObjectId>>>,
}

type ObjectIdsPtr = Arc<ObjectIds>;

impl std::default::Default for ObjectIds {
    fn default() -> Self {
        Self { ids: RwLock::new(BTreeSet::new()) }
    }
}

impl ObjectIds {

    pub(super) fn insert(&self, id: ObjectId) -> bool {
        let id = DynamicPtr::new(id);
        let ids = &mut *self.ids.write().unwrap();
        if ids.contains(&id) == false {
            ids.insert(id)
        } else {
            true
        }
    }

    pub(super) fn insert_ids(&self, id_array: Vec<ObjectId>) {
        let ids = &mut *self.ids.write().unwrap();

        for id in id_array {
            let id = DynamicPtr::new(id);
            if ids.contains(&id) {
                ids.insert(id);
            }
        }
    }

    pub(super) fn collect(&self) -> Vec<ObjectIdPtr> {
        let mut ids = vec![];
        self.ids.read().unwrap()
            .iter()
            .for_each(| id | {
                ids.push(id.clone());
            });
        ids
    }
}

#[derive(Clone)]
pub struct MessageCenter(Arc<MessageCenterImpl>);

impl MessageCenter {
    pub fn new(manager: Manager) -> Self {
        Self(Arc::new(MessageCenterImpl{
            manager,
            subscribe_message: RwLock::new(BTreeMap::new()),
        }))
    }

    pub fn subscribe(&self, id: ObjectId, topic: TopicRef) -> NearResult<()> {

        match self.create_message(&topic)
                  .insert(id) {
            true => { info!("Subscribe {} to {}", topic, id) },
            false => { info!("Unsubscribe {} to {}", topic, id) },
        }

        Ok(())
    }

    fn create_message(&self, topic: &TopicRef) -> ObjectIdsPtr {
        let topic_secondary = topic.secondary().unwrap_or("");

        let messages = &mut *self.0.subscribe_message.write().unwrap();
        match messages.get(topic_secondary) {
            Some(ids) => { ids.clone() }
            None => {
                let ids = Arc::new(ObjectIds::default());
                messages.insert(topic_secondary.to_string(), ids);
                ids
            }
        }

    }
}

impl MessageTrait for MessageCenter {
    fn clone_as_message(&self) -> Box<dyn MessageTrait> {
        Box::new(self.clone())
    }

    fn is_core_message(&self) -> bool {
        false
    }

    fn primary_label(&self) -> &str {
        unimplemented!()
    }
}

impl ProcessTrait for MessageCenter {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, from: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        unimplemented!()
    }
}
