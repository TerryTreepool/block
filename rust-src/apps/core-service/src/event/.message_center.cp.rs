
use std::{sync::{Arc, RwLock, }, 
          collections::{BTreeMap, btree_map::Entry, BTreeSet}, 
    };

use near_base::{NearResult, ObjectId, DynamicPtr};
use near_transport::{ProcessTrait, RoutineEventTrait};

use super::{Manager, MessageTrait, Topic};

struct MessageCenterImpl {
    manager: Manager,
    subscribe_message: RwLock<BTreeMap<(String, Option<String>), ObjectIds>>,
}

type ObjectIdPtr = DynamicPtr<ObjectId>;

struct ObjectIds {
    ids: RwLock<BTreeSet<ObjectIdPtr>>,
}

impl std::default::Default for ObjectIds {
    fn default() -> Self {
        Self { ids: RwLock::new(BTreeSet::new()) }
    }
}

impl ObjectIds {
    pub(super) fn insert(&self, id: ObjectId) -> bool {
        let id_ptr = ObjectIdPtr::new(id);
        let ids = &mut *self.ids.write().unwrap();
        if ids.contains(&id_ptr) == false {
            ids.insert(id_ptr)
        } else {
            true
        }
    }

    pub(super) fn insert_ids(&self, id_array: &[ObjectId]) {
        let ids = &mut *self.ids.write().unwrap();

        id_array.iter()
                .for_each(| id | {
                    let id = ObjectIdPtr::new(id.clone());
                    if ids.contains(&id) == false {
                        ids.insert(id);
                    }
                })
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

    pub fn subscribe_message(&self, id: ObjectId, topic: Topic<'_>) -> NearResult<()> {
        let topic = topic.primary_secondary_label();

        let messages = &mut *self.0.subscribe_message.write().unwrap();

        match messages.entry(topic) {
            Entry::Occupied(exist) => {
                exist.get().insert(id);
            }
            Entry::Vacant(entry) => {
                let ids = ObjectIds::default();
                ids.insert(id);
                entry.insert(ids);
            }
        }

        Ok(())
    }
}

// impl MessageTrait for MessageCenter {
//     fn clone_as_message(&self) -> Box<dyn MessageTrait> {
//         Box::new(self.clone())
//     }

//     fn is_core_message(&self) -> bool {
//         false
//     }

//     fn message_primary_label(&self) -> &str {
//         ""
//     }
// }

impl ProcessTrait for MessageCenter {
    fn create_routine(&self, from: &ObjectId, topic: &str) -> NearResult<Box<dyn RoutineEventTrait>> {
        unimplemented!()
    }
}
