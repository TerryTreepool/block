
use std::sync::atomic::AtomicU64;

use near_base::now;
use topic_util::types::thing_data::ThingId;

use super::{ScheduleTrait, OnSchedultEventTrait};

pub struct CycleTimeComponents {
    now: AtomicU64,
    cycle_time: u64,
}

impl std::fmt::Display for CycleTimeComponents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cycle {}(us) time", self.cycle_time)
    }
}

impl CycleTimeComponents {

    pub fn new(cycle_time: std::time::Duration) -> Self {
        Self { 
            now: AtomicU64::new(now()),
            cycle_time: cycle_time.as_micros() as u64
        }
    }

}

#[async_trait::async_trait]
impl ScheduleTrait<()> for CycleTimeComponents {
    fn update_schedule(&self, _: ()) {
    }

    fn remove_schedule(&self, _: Vec<ThingId>) {
    }

    async fn execute<E: OnSchedultEventTrait>(&self, event: E) -> near_base::NearResult<()> {
        let now = now();

        if now - self.now.load(std::sync::atomic::Ordering::SeqCst) > self.cycle_time {
            self.now.store(now, std::sync::atomic::Ordering::SeqCst);
            event.on_event(Default::default()).await
        } else {
            Ok(())
        }
    }

    async fn release(&self) {
    }

}
