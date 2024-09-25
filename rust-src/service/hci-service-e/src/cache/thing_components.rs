
use std::{sync::{Arc, RwLock}, collections::{BTreeMap, btree_map::Entry}, str::FromStr, borrow::BorrowMut, };

use mac_address::MacAddress;

use near_base::{ObjectId, thing::ThingObject, NearResult, NearError, ErrorCode, now};

use crate::{lua::data::Data, tasks::{TaskCbTrait, TaskModule, TaskCbData}};

use super::ThingStatus;

struct ThingComponent {
    thing: ThingObject,
    status: ThingStatus,
}

#[derive(Clone)]
pub struct ThingComponentPtr(Arc<ThingComponent>);

impl From<ThingComponent> for ThingComponentPtr {
    fn from(value: ThingComponent) -> Self {
        Self(Arc::new(value))
    }
}

impl ThingComponentPtr {
    pub fn new(thing: ThingObject) -> Self {
        Self(Arc::new(ThingComponent {
            thing, 
            status: ThingStatus::Online(now(), Default::default()), 
        }))
    }

    #[inline]
    pub fn thing(&self) -> &ThingObject {
        &self.0.thing
    }

    #[inline]
    pub fn status(&self) -> ThingStatus {
        self.0.status.clone()
    }

    pub fn online(&mut self, data: Data) {
        let mut_self = unsafe { &mut *(Arc::as_ptr(&self.0) as *mut ThingComponent) };
        let status = &mut mut_self.status;
        match status {
            ThingStatus::Offline(_, _) | ThingStatus::Online(_, _) => { *status = ThingStatus::Online(now(), data); }
            _ => { /* ignore */ }
        }
    }

    pub fn offline(&mut self) {
        let mut_self = unsafe { &mut *(Arc::as_ptr(&self.0) as *mut ThingComponent) };
        let status = &mut mut_self.status;
        match status {
            ThingStatus::Offline(_, _) => { /* ignore */ }
            ThingStatus::Online(_, data) => { *status = ThingStatus::Offline(now(), data.take_map().into()); }
            _ => { /* ignore */ }
        }
    }

    pub fn disable(&mut self) {
        unsafe { &mut *(Arc::as_ptr(&self.0) as *mut ThingComponent) }
            .status = ThingStatus::Disable;
    }

    pub fn enable(&mut self) {
        unsafe { &mut *(Arc::as_ptr(&self.0) as *mut ThingComponent) }
            .status = ThingStatus::Offline(now(), Default::default());
    }
}

#[derive(Default)]
struct ThingCollectImpl {
    things: Vec<ThingComponentPtr>,
    things_id_mapping: BTreeMap<ObjectId, ThingComponentPtr>,
    things_mac_mapping: BTreeMap<MacAddress, ThingComponentPtr>,
}

#[derive(Default)]
pub struct ThingCollect(RwLock<ThingCollectImpl>);

impl ThingCollect {

    pub fn add_things(&self, things: impl Iterator<Item=ThingObject>) {
        let w = &mut *self.0.write().unwrap();

        let mut add_thing = | thing: ThingObject | {
            let mac_address = thing.desc().content().mac_address().clone().into();
            let thing_id = thing.object_id().clone();
    
            let thing = ThingComponentPtr::new(thing);

            let newly = {
                match w.things_id_mapping.entry(thing_id) {
                    Entry::Vacant(empty) => {
                        empty.insert(thing.clone());
                        true
                    }
                    Entry::Occupied(mut exist) => {
                        exist.get_mut().enable();
                        false
                    }
                }
            };

            if newly {
                let _ = w.things_mac_mapping.insert(mac_address, thing.clone());
    
                w.things.push(thing);
            }
        };
    
        for thing in things {
            let _ = add_thing(thing);
        }
    }

    pub fn remove_thing(&self, thing_id: &ObjectId) {
        let w = &mut *self.0.write().unwrap();

        if let Some(thing_info) = w.things_id_mapping.get_mut(thing_id) {
            thing_info.disable();
        }
    }

    pub fn get_all_thing(&self) -> Vec<ThingComponentPtr> {
        {
            self.0
                .read().unwrap()
                .things
                .clone()
        }
        .into_iter()
        .filter(| thing_c | {
            match thing_c.status() {
                ThingStatus::Disable => false,
                _ => true
            }
        })
        .collect()
    }

    #[allow(unused)]
    pub fn get_thing_by_mac(&self, mac: [u8; 6]) -> NearResult<ThingComponentPtr> {
        let mac = mac.into();
        self.0
            .read().unwrap()
            .things_mac_mapping
            .get(&mac)
            .cloned()
            .ok_or(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("[{}] not found mac.", mac)))
    }

    #[allow(unused)]
    pub fn get_thing_by_id(&self, id: &str) -> NearResult<ThingComponentPtr> {
        let id = ObjectId::from_str(id)?;
        self.0
            .read().unwrap()
            .things_id_mapping
            .get(&id)
            .cloned()
            .ok_or(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("[{}] not found id.", id)))
    }

    pub fn offline<'a>(&self, thing_ids: impl Iterator<Item=&'a ObjectId>) {
        let w = &mut *self.0.write().unwrap();

        for thing_id in thing_ids {
            if let Some(thing) = w.things_id_mapping.get_mut(thing_id) {
                thing.borrow_mut().offline();
            }
        }
    }

}

#[async_trait::async_trait]
impl TaskCbTrait for ThingCollect {
    async fn on_taskcb(&self, task_module: TaskModule, data: TaskCbData) {
        debug_assert!(task_module == TaskModule::QueryThing);

        let (mac, data) = data.split();

        let w = &mut *self.0.write().unwrap();

        if let Some(thing) = w.things_mac_mapping.get_mut(&mac) {
            thing.online(data);
        }
    }
}
