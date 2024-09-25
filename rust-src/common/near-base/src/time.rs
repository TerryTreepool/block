
use std::time::{SystemTime, UNIX_EPOCH};

pub type Timestamp = u64;

static _TIME_TTO_MICROSECONDS_OFFSET: u64 = 11644473600_u64 * 1000 * 1000;

fn unix_timestamp(time: &SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).unwrap().as_micros() as u64 + _TIME_TTO_MICROSECONDS_OFFSET
}

pub fn now() -> u64 {
    unix_timestamp(&SystemTime::now())
}
