
use std::sync::{Arc, RwLock};

use near_base::{device::DeviceId, Timestamp, now, SequenceValue};

#[derive(Clone, Debug)]
pub enum PeerStatusKind {
    Connecting(Timestamp /* start timestamp */),
    Online(Timestamp /* start timestamp */, Timestamp /* online timestamp */),
}

struct PeerStatusImpl {
    peer_id: DeviceId,
    status: PeerStatusKind,

    // records: BTreeMap<(DeviceId, TempSeq), StatusKind>,
    // will_cache_record: Vec<(Option<DeviceId>, Option<TempSeq>, StatusKind)>,
}

pub struct PeerStatus(Arc<RwLock<PeerStatusImpl>>);

impl PeerStatus {
    pub fn new(peer_id: DeviceId) -> Self {
        Self(Arc::new(RwLock::new(
            PeerStatusImpl {
                peer_id,
                status: PeerStatusKind::Connecting(now()),
            }
        )))
    }

    pub fn online(&self, _seq: SequenceValue, send_time: Timestamp) {
        let status = &mut *self.0.write().unwrap();

        match status.status {
            PeerStatusKind::Online(_start_stamp, _online_stamp) => {}
            PeerStatusKind::Connecting(start_stamp) => {
                status.status = PeerStatusKind::Online(start_stamp, send_time);
            }
        }
    }

}
