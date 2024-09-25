
pub mod manager;

use std::sync::{atomic::{AtomicU64, AtomicPtr}, Arc};

use near_base::Timestamp;

#[derive(Default)]
pub struct Stat {
    pub duration: Timestamp,
    pub rate_mbps: u64,
    pub max_mbps: u64,
    pub min_mbps: u64,
}

#[derive(Default)]
struct BytePerfStat {
    begin_time_stamp_ms: Timestamp, // time since the UDT entity is started, in milliseconds
    total_bytes: u64,               // total number of data bytes
    curr_bytes: AtomicU64,
    rate_mbps: AtomicU64,           // sending rate in Mb/s
    max_mbps: AtomicU64,
    min_mbps: AtomicU64,
}

#[derive(Clone)]
pub struct BytePerfStatPtr(Arc<BytePerfStat>);

impl BytePerfStatPtr {
    pub fn new(now: Timestamp, total_bytes: u64) -> BytePerfStatPtr {
        BytePerfStatPtr(Arc::new(BytePerfStat {
            begin_time_stamp_ms: now,
            total_bytes,
            ..Default::default()
        }))
    }

    pub fn update(&self, when: Timestamp, bytes: u64) -> Stat {
        debug_assert!(self.0.begin_time_stamp_ms <= when);

        let duration = when - self.0.begin_time_stamp_ms;
        let finished_bytes = self.0.curr_bytes.fetch_add(bytes, std::sync::atomic::Ordering::SeqCst);

        if duration > 0 {
            let rate_mbps = finished_bytes / duration;

            self.0.rate_mbps.store(rate_mbps, std::sync::atomic::Ordering::SeqCst);

            Stat {
                duration,
                rate_mbps,
                ..Default::default()
            }
        } else {
            Stat::default()
        }
    }
}
