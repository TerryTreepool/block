
use std::{sync::atomic::AtomicUsize, time::Duration};

use once_cell::sync::OnceCell;

pub struct SearchTask {
    times: AtomicUsize,
    interval: Duration,
}

impl std::default::Default for SearchTask {
    fn default() -> Self {
        Self {
            times: AtomicUsize::new(0),
            interval: Duration::from_secs(1),
        }
    }
}

impl SearchTask {
    pub fn get_instance() -> &'static SearchTask {
        static INSTANCE: OnceCell<SearchTask> = OnceCell::new();

        INSTANCE.get_or_init(|| {
            let r = SearchTask::default();
            r
        })
    }

    pub fn add_task(&self) {}
}
