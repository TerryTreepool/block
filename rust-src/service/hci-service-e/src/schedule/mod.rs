
pub mod manager;

use near_base::{thing::ThingObject, NearResult, ObjectId};

#[async_trait::async_trait]
pub(super) trait ScheduleTrait: Send + Sync {
    fn add_schedule(&self, thing: ThingObject) -> NearResult<()>;
    fn remove_schedule(&self, thing_id: &ObjectId) -> NearResult<()>;

    async fn init_schedule(&self) -> near_base::NearResult<()>;
    async fn on_schedule(&self);
}

#[derive(Default)]
pub struct Config {
    query_schedule_config: QueryScheduleConfig,
}

#[derive(Clone)]
pub struct QueryScheduleConfig {
    pub(crate) interval: std::time::Duration,
    pub(crate) timeout_response: std::time::Duration,

}

impl std::default::Default for QueryScheduleConfig {
    fn default() -> Self {
        Self {
            interval: std::time::Duration::from_secs(60),
            timeout_response: std::time::Duration::from_secs(120),
        }
    }
}
