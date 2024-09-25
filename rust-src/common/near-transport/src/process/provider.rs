
use std::{sync::{RwLock, Arc, },
          collections::{BTreeMap, hash_map::DefaultHasher}, convert::TryFrom, hash::Hasher, 
    };

use log::debug;
use near_base::{sequence::SequenceString, NearError, NearResult, ObjectId, Timestamp 
    };

use crate::HeaderMeta;

use super::{Routine, 
            itf::ItfTrait, 
            {EventResult, ResponseEvent, TransferEvent}, RoutineWrap,
    };

pub type EventTextResult = EventResult<Vec<u8>>;

#[async_trait::async_trait]
pub trait RoutineEventTrait: Send + Sync {
    async fn emit(&self, header_meta: &HeaderMeta, data: Vec<u8>) -> NearResult<EventTextResult>;
}

// impl<RESP: ItfTrait> TryFrom<EventResult<RESP>> for EventTextResult {
//     type Error = NearError;

//     fn try_from(event: EventResult<RESP>) -> Result<Self, Self::Error> {
//         match event {
//             EventResult::<RESP>::Response(event) => EventTextResult::try_from(event),
//             EventResult::<RESP>::Transfer(event) => EventTextResult::try_from(event),
//         }
//     }
// }

#[async_trait::async_trait]
impl<REQ, RESP> RoutineEventTrait for RoutineWrap<REQ, RESP>
where REQ: ItfTrait,
      RESP: ItfTrait {
    async fn emit(&self, header_meta: &HeaderMeta, data: Vec<u8>) -> NearResult<EventTextResult> {
        self.0.emit(header_meta, data).await
    }

}

impl<RESP: ItfTrait> TryFrom<ResponseEvent<RESP>> for EventTextResult {
    type Error = NearError;

    fn try_from(event: ResponseEvent<RESP>) -> Result<Self, Self::Error> {
        let resp = &event.data;
        let mut v: Vec<u8> = vec![0u8; resp.raw_capacity()];
        resp.serialize(v.as_mut_slice())?;
        Ok(EventTextResult::Response(ResponseEvent { data: v } ))
    }

}

impl<RESP: ItfTrait> TryFrom<TransferEvent<RESP>> for EventTextResult {
    type Error = NearError;

    fn try_from(event: TransferEvent<RESP>) -> Result<Self, Self::Error> {
        let resp = event.data;
        let data = {
            let mut v: Vec<u8> = Vec::with_capacity(resp.raw_capacity());
            resp.serialize(v.as_mut_slice())?;
            v
        };

        Ok(EventTextResult::Transfer(TransferEvent {
            to: event.to,
            topic: event.topic,
            data,
        }))
    }
}

struct RoutineImpl<REQ, RESP>(Box<dyn Routine<REQ, RESP>>)
where REQ: ItfTrait,
      RESP: ItfTrait;

#[async_trait::async_trait]
impl<REQ, RESP> RoutineEventTrait for dyn Routine<REQ, RESP>
where REQ: ItfTrait,
      RESP: ItfTrait {
    async fn emit(&self, header_meta: &HeaderMeta, data: Vec<u8>) -> NearResult<EventTextResult> {
        let (req, _) = REQ::deserialize(&data)?;

        let r = match self.on_routine(header_meta, req).await {
            EventResult::<RESP>::Response(resp) => {
                debug!("response: header_meta: {}", header_meta);
                EventTextResult::try_from(resp)
            }
            EventResult::<RESP>::Transfer(resp) => {
                debug!("transfer: header_meta: {}", header_meta);
                EventTextResult::try_from(resp)
            }
            EventResult::<RESP>::Ignore => Ok(EventTextResult::Ignore),
        }?;

        Ok(r)
    }

}

#[async_trait::async_trait]
impl<REQ, RESP> RoutineEventTrait for RoutineImpl<REQ, RESP>
where REQ: ItfTrait + std::fmt::Display,
      RESP: ItfTrait + std::fmt::Display {
    async fn emit(&self, header_meta: &HeaderMeta, data: Vec<u8>) -> NearResult<EventTextResult> {

        self.0.emit(header_meta, data).await

    }

}

pub struct RoutineEventCache {
    // The Sender is the same as the request's sender
    sender: ObjectId,
    routine: Box<dyn RoutineEventTrait>,
}

impl From<(ObjectId, Box<dyn RoutineEventTrait>)> for RoutineEventCache {
    fn from(cx: (ObjectId, Box<dyn RoutineEventTrait>)) -> Self {
        let (sender, routine) = cx;

        Self {
            sender, routine: routine,
        }
    }
}

impl RoutineEventCache {
    fn split(self) -> (ObjectId, Box<dyn RoutineEventTrait>) {
        (self.sender, self.routine)
    }
}

struct RoutineIDBuilder<'b> {
    pub(crate) sequence:  &'b SequenceString,
    pub(crate) requestor: &'b ObjectId,
    pub(crate) timestamp: Timestamp,
}

impl RoutineIDBuilder<'_> {
    pub fn build(self) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write(self.sequence.as_ref());
        hasher.write(self.requestor.as_ref());
        hasher.write(self.timestamp.to_be_bytes().as_slice());

        hasher.finish()
    }
}

struct EventManagerImpl {
    // routines: RwLock<BTreeMap<u16, Box<dyn RoutineEventTrait>>>,
    // routines: RwLock<BTreeMap<(ObjectId, u32), RoutineEventCache>>,
    routines: RwLock<BTreeMap<u64, RoutineEventCache>>,
}

#[derive(Clone)]
pub struct EventManager(Arc<EventManagerImpl>);

impl EventManager {
    pub fn new() -> Self {
        Self(Arc::new(EventManagerImpl {
            routines: RwLock::new(BTreeMap::new()),
        }))
    }

    pub fn add_routine<REQ, RESP>(&self, requestor: &ObjectId, sequence: &SequenceString, timestamp: Timestamp, routine: RoutineEventCache) -> NearResult<()>
    where REQ: ItfTrait,
          RESP: ItfTrait {
        let routine_id = RoutineIDBuilder{
            requestor,
            sequence,
            timestamp,
        }.build();

        let routines = &mut *self.0.routines.write().unwrap();

        routines.entry(routine_id)
            .or_insert({
                routine
            });

        Ok(())
    }

    pub fn join_routine(&self, requestor: &ObjectId, sequence: &SequenceString, timestamp: Timestamp, routine: RoutineEventCache) -> NearResult<()> {
        let routine_id = RoutineIDBuilder{
            requestor,
            sequence,
            timestamp,
        }.build();

        let routines = &mut *self.0.routines.write().unwrap();

        routines.entry(routine_id)
            .or_insert({
                routine
            });

        Ok(())
    }

    pub fn take_routine(&self, requestor: &ObjectId, sequence: &SequenceString, timestamp: Timestamp) -> Option<(ObjectId /* sender */, Box<dyn RoutineEventTrait> /* routine */)> {
        let routine_id = RoutineIDBuilder{
            requestor,
            sequence,
            timestamp,
        }.build();

        self.0.routines.write().unwrap()
            .remove(&routine_id)
            .map(| routine_cache | routine_cache.split())
    }

}

// #[async_trait::async_trait]
// impl PackageEventTrait for EventManager {
//     async fn on_package_event(&self, from: ObjectId, command: u16, sequence: u32, data: &[u8]) -> NearResult<Option<Vec<u8>>> {
//         let routine = match self.0.routines.read().unwrap().remove(&command) {
//             Some(routine) => { routine },
//             None => { 
//                 return Err(NearError::new(ErrorCode::NEAR_ERROR_TOPIC_UNKNOWN, format!("[{}] command not found.", command)));
//             }
//         };

//         routine.emit(data).await
//     }
// }

// #[async_trait::async_trait]
// impl<REQ, RESP> Routine<REQ, RESP> for EventManager<REQ, RESP>
// where REQ: ItfTrait,
//       RESP: ItfTrait, {
//     fn clone_as_routine(&self) -> Box<dyn Routine<REQ, RESP>> {
//         unimplemented!()
//     }

//     async fn on_routine(&self, command: u16, param: &REQ) -> NearResult<RESP> {
//         if let Some(routine) = self.routine_of(command) {

//         }
//     }

// }
