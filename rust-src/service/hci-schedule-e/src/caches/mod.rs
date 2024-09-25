
pub mod schedule_manager;

mod schedule_data;
mod schedule_cycletime;

#[derive(Clone)]
pub enum ThingStatus {
    Offline(near_base::Timestamp),
    Online(near_base::Timestamp, ()),
    Disable,
}

impl std::fmt::Display for ThingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Offline(_) => write!(f, "offline"),
            Self::Online(_, _) => write!(f, "online"),
            Self::Disable => write!(f, "disable"),
        }
    }
}

use std::sync::Arc;

use futures::Future;
use near_base::NearResult;
use topic_util::types::thing_data::{ThingId, ThingData};

#[async_trait::async_trait]
pub(crate) trait ScheduleTrait<P>: Send + Sync {

    fn update_schedule(&self, p: P);
    fn remove_schedule(&self, things: Vec<ThingId>);

    async fn execute<E: OnSchedultEventTrait>(&self, event: E) -> near_base::NearResult<()>;
    async fn release(&self);
}

pub(crate) type ScheduleTraitRef<S> = Arc<S>;

#[async_trait::async_trait]
impl<P, S: ScheduleTrait<P>> ScheduleTrait<P> for ScheduleTraitRef<S> {

    fn update_schedule(&self, p: P) {
        self.as_ref().update_schedule(p)
    }

    fn remove_schedule(&self, things: Vec<ThingId>) {
        self.as_ref().remove_schedule(things);
    }

    async fn execute<E: OnSchedultEventTrait>(&self, event: E) -> near_base::NearResult<()> {
        self.as_ref().execute(event).await
    }

    async fn release(&self) {
        self.as_ref().release().await
    }

}

#[async_trait::async_trait]
pub trait OnSchedultEventTrait: Send + Sync {
    async fn on_event(&self, thing_dataes: Vec<(ThingId, ThingData)>) -> near_base::NearResult<()>;
}

#[async_trait::async_trait]
impl<F, Fut> OnSchedultEventTrait for F
where
    F: Send + Sync + 'static + Fn(Vec<(ThingId, ThingData)>) -> Fut,
    Fut: Future<Output = NearResult<()>> + Send + 'static,
    {
        async fn on_event(&self, thing_dataes: Vec<(ThingId, ThingData)>) -> near_base::NearResult<()> {
            let fut = (self)(thing_dataes);
            fut.await
        }
    }
