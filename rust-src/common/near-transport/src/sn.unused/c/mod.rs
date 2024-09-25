
pub mod ping;

mod call;
mod task;

use std::time::Duration;

#[derive(Copy, Clone)]
pub struct Config {
    pub min_random_vport: u16,
    pub max_random_vport: u16,
    pub max_try_random_vport_times: usize,

    pub ping_interval_connect: Duration,
    pub ping_interval: Duration,
    pub offline: Duration,

    pub call_interval: Duration,
    pub call_timeout: Duration,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            min_random_vport: 32767,
            max_random_vport: 65535,
            max_try_random_vport_times: 1,

            ping_interval_connect: Duration::from_secs(30),
            ping_interval: Duration::from_millis(25000),
            offline: Duration::from_secs(60),
            call_interval: Duration::from_millis(200),
            call_timeout: Duration::from_millis(3000),
        }
    }
}
