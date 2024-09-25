
use std::{collections::{btree_map::Entry, BTreeMap}, sync::{Arc, Mutex, RwLock}, task::Waker};

use near_base::{ErrorCode, NearError, NearResult, ObjectId};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct CallTag {
    sn_id: ObjectId,
    peer_id: ObjectId,
    call_sequence: u32,
}

struct CallCenterManagerImpl {
    entries: RwLock<BTreeMap<CallTag, CallResultRef>>,
}

pub struct CallCenterManager(Arc<CallCenterManagerImpl>);

impl CallCenterManager {

    pub fn get_instance() -> &'static CallCenterManager {
        static INSTANCE: once_cell::sync::OnceCell<CallCenterManager> = once_cell::sync::OnceCell::new();

        INSTANCE.get_or_init(|| {
            Self::new()
        })
    }

    fn new() -> Self {
        Self(Arc::new(CallCenterManagerImpl {
            entries: RwLock::new(BTreeMap::new()),
        }))
    }

    pub fn append_result(
        &self, 
        sn_id: ObjectId, peer_id: ObjectId, call_sequence: u32, 
        res: CallResultRef
    ) -> NearResult<()> {
        match self.0.entries.write().unwrap().entry(CallTag { sn_id, peer_id, call_sequence }) {
            Entry::Occupied(_) => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_ALREADY_EXIST, "already exist."))
            }
            Entry::Vacant(empty) => {
                empty.insert(res);
                Ok(())
            }
        }

    }

    pub async fn call_result(
        &self, 
        sn_id: ObjectId,
        peer_id: ObjectId, 
        call_sequence: u32,
        status: CallSessionStatus,
    ) {
        let f = 
            match self.0.entries
                        .write().unwrap()
                        .remove(&CallTag{ sn_id, peer_id, call_sequence }) {
                Some(fut) => fut.clone(),
                None => return
            };

         let waker = {
            let result = &mut *f.0.lock().unwrap();

            result.found = Some(CallSession::with_status(status));
            result.waker.take()
        };

        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

pub enum CallSessionStatus {
    Connecting,
    Established,
    Closed(ErrorCode),
}

pub struct CallSession {
    status: CallSessionStatus,
}

impl CallSession {
    pub fn new() -> CallSession {
        Self {
            status: CallSessionStatus::Connecting,
        }
    }

    pub fn with_status(status: CallSessionStatus) -> Self {
        Self {
            status,
        }
    }

    #[inline]
    pub fn status(&self) -> &CallSessionStatus {
        &self.status
    }
}

struct CallResult {
    found: Option<CallSession>,
    waker: Option<Waker>,
}

// struct CallResult {
//     sessions: CallSession,
//     result: Mutex<CallResult>,
// }

#[derive(Clone)]
pub struct CallResultRef(Arc<Mutex<CallResult>>);

impl CallResultRef {
    pub fn new(sn_id: ObjectId, peer_id: ObjectId, call_sequeuce: u32) -> NearResult<Self> {
        let f = Self(Arc::new(Mutex::new(CallResult {
            found: None,
            waker: None,
        })));

        CallCenterManager::get_instance()
            .append_result(sn_id, peer_id, call_sequeuce, f.clone())?;

        Ok(f)
    }

}

impl std::future::Future for CallResultRef {
    type Output = CallSession;

    fn poll(
        self: std::pin::Pin<&mut Self>, 
        cx: &mut std::task::Context<'_>
    ) -> std::task::Poll<Self::Output> {
        let result = &mut *self.0.lock().unwrap();

        match result.found.as_mut() {
            Some(found) => {
                let mut ret = CallSession::new();
                std::mem::swap(found, &mut ret);
                std::task::Poll::Ready(ret)
            }
            None => {
                result.waker = Some(cx.waker().clone());
                std::task::Poll::Pending
            }
        }
    }
}
