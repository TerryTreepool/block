
pub mod add;
pub mod remove;
pub mod execute;

use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};

pub(super) struct ScheduleId<'a> {
    schedule_id: &'a str,
}

impl<'a> ScheduleId<'a> {
    pub(super) fn new(schedule_id: &'a str) -> Self {
        Self {
            schedule_id,
        }
    }

    pub(super) fn to_u64(self) -> u64{
        let mut hasher = DefaultHasher::new();
        self.schedule_id.hash(&mut hasher);
        hasher.finish()
    }
}
