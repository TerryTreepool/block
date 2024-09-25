
use std::{sync::RwLock, collections::{BTreeMap, btree_map::Entry}};

use topic_util::types::thing_data::{ThingData, ThingId};

use super::{ScheduleTrait, OnSchedultEventTrait};

#[derive(Default)]
pub struct ScheduleData {
    things: RwLock<BTreeMap<ThingId, ThingData>>,
}

#[async_trait::async_trait]
impl ScheduleTrait<Vec<(ThingId, ThingData)>> for ScheduleData {

    fn update_schedule(&self, things: Vec<(ThingId, ThingData)>) {
        let w = &mut *self.things.write().unwrap();

        for (thing, mut thing_data) in things {
            match w.entry(thing) {
                Entry::Occupied(mut exist) => {
                    exist.get_mut().swap(thing_data.take_map())
                }
                Entry::Vacant(empty) => {
                    empty.insert(thing_data.into());
                }
            }
        }
    }

    fn remove_schedule(&self, things: Vec<ThingId>) {
        let w = &mut *self.things.write().unwrap();

        for thing in  things {
            w.remove(&thing);
        }
    }

    async fn execute<E: OnSchedultEventTrait>(&self, event: E) -> near_base::NearResult<()> {

        let thing_data: Vec<(ThingId, ThingData)> = {
            self.things
                .read().unwrap()
                .iter()
                .map(| (k, v) | {
                    (k.clone(), v.clone())
                })
                .collect()
        };

        event.on_event(thing_data).await
    }

    async fn release(&self) {
    }

}
