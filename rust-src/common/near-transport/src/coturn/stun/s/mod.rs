
pub mod peer_manager;

use std::time::Duration;

#[derive(Clone)]
pub struct Config {
    pub polling_interval: Duration,

    pub offline: Duration,

    pub knock_timeout: Duration,    // default is 5 minutes

    pub invite_interval: Duration,
    pub invite_timeout: Duration,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            polling_interval: Duration::from_micros(100000),
            offline: Duration::from_millis(300000),
            knock_timeout: Duration::from_secs(300),
            invite_interval: Duration::from_millis(200),
            invite_timeout: Duration::from_secs(30),
        }
    }
}
