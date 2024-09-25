
mod s;
mod c;
mod p;

pub(crate) use s::Service as TurnService;
pub(crate) use c::task::Task as TurnTask;
// pub(crate) use c::key::*;

#[derive(Clone)]
pub struct Config {
    pub keepalive: std::time::Duration,
    pub mixhash_live_minutes: u8,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            keepalive: std::time::Duration::from_secs(60),
            mixhash_live_minutes: 31,
        }
    }
}
