
use near_base::{Timestamp, NearResult};

pub trait OnTimeTrait {
    fn on_time_escape(&self, now: Timestamp);
}

#[async_trait::async_trait]
pub trait OnBuildPackage<DataContext, R> {
    async fn build_package(&self, data: DataContext) -> NearResult<R>;
}
