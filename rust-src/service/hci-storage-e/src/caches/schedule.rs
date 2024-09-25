
use near_base::{Serialize, Deserialize, hash_data, NearResult, NearError, ErrorCode, now};

use protos::hci::schedule::*;
use storage::ItemTrait;
use topic_util::types::Status;


struct ScheduleIdBuilder<'a> {
    pub schedule_name: &'a str,
    pub now: near_base::Timestamp,
}

impl ScheduleIdBuilder<'_> {
    pub(super) fn build(self) -> String {
        let buf = {
            let mut buf = vec![0u8; self.schedule_name.raw_capacity() + self.now.raw_capacity()];

            let _end = self.schedule_name.serialize(&mut buf).unwrap();
            let _end = self.now.serialize(_end).unwrap();

            buf
        };

        let binding = hash_data(buf.as_slice());
        let buf = binding.as_slice();
        // let buf = binding.as_ref().as_ref();
        hex::encode_upper(&buf[0..16])
    }
}


#[derive(Clone)]
pub struct ScheduleItem {
    schedule: Schedule_info,
}

impl ScheduleItem {
    pub fn create_new(schedule_name: String, schedule_img_idx: u32, schedule_mode: Schedule_mode) -> NearResult<Self> {
        let schedule_name = schedule_name.as_str().trim();
        if schedule_name.is_empty() {
            Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "schedule name can't empty."))
        } else {
            Ok(())
        }?;

        Ok(Self {
            schedule: Schedule_info {
                schedule_id: ScheduleIdBuilder{
                    schedule_name,
                    now: now(),
                }.build(), 
                schedule_name: schedule_name.to_owned(),
                schedule_img_idx,
                status: Status::Eanbled.into(),
                mode: schedule_mode.into(),
                ..Default::default()
        }})
    }

    pub fn enable(&mut self) {
        self.schedule.set_status(Status::Eanbled.into());
    }

    pub fn disable(&mut self) {
        self.schedule.set_status(Status::Disabled.into());
    }

    pub fn update_name(&mut self, new_schedule_name: String) {
        self.schedule.set_schedule_name(new_schedule_name);
    }

    pub fn update_img_index(&mut self, schedule_img_idx: u32) {
        self.schedule.set_schedule_img_idx(schedule_img_idx);
    }

    pub fn update_timeperiod_mode(&mut self, timeperiod_mode: Schedule_timeperiod_mode) {
        self.schedule.set_timeperiod_mode(timeperiod_mode);
    }

    pub fn update_condition_mode(&mut self, condition_mode: Schedule_condition_mode) {
        self.schedule.set_condition_mode(condition_mode);
    }

    pub fn insert_relation(&mut self, mut new_relation: Schedule_relation_info) -> bool {
        if let Some(mut_relation) = 
            self.schedule.mut_thing_relation().iter_mut().find(| relation | relation.thing_id() == new_relation.thing_id()) {
            std::mem::swap(mut_relation, &mut new_relation);
        } else {
            self.schedule.mut_thing_relation().push(new_relation);
        }

        true
    }

    pub fn remove_relation(&mut self, remove_iter: &str) {
        let remain_relations = 
            self.schedule
                .take_thing_relation()
                .into_iter()
                .filter(| relation | {
                    !(relation.thing_id() == remove_iter)
                })
                .collect();

        self.schedule.set_thing_relation(remain_relations);
    }
    // pub fn insert_thing(&mut self, thing_ids: impl Iterator<Item=String>) {
    //     for thing_id in thing_ids {
    //         if !self.thing_ids().contains(&thing_id) {
    //             self.mut_thing_ids().push(thing_id);
    //         }
    //     }
    // }

    // pub fn remove_thing(&mut self, thing_ids: &[String]) {
    //     let things = 
    //         self.take_thing_ids()
    //             .into_iter()
    //             .filter(| thing_id | {
    //                 !thing_ids.contains(thing_id)
    //             })
    //             .collect();

    //     self.set_thing_ids(things);
    // }
}

impl From<ScheduleItem> for Schedule_info {
    fn from(value: ScheduleItem) -> Self {
        value.schedule
    }
}

impl ItemTrait for ScheduleItem {
    fn id(&self) -> &str {
        self.schedule.schedule_id()
    }
}

impl Serialize for ScheduleItem {
    fn raw_capacity(&self) -> usize {
        self.schedule.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> near_base::NearResult<&'a mut [u8]> {
        self.schedule.serialize(buf)
    }
}

impl Deserialize for ScheduleItem {
    fn deserialize<'de>(buf: &'de [u8]) -> near_base::NearResult<(Self, &'de [u8])> {
        let (schedule, buf) = Schedule_info::deserialize(buf)?;

        Ok((Self{
            schedule: schedule.into(),
        }, buf))
    }
}

impl std::ops::Deref for ScheduleItem {
    type Target = Schedule_info;

    fn deref(&self) -> &Self::Target {
        &self.schedule
    }
}
