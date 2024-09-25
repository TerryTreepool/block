
use std::{sync::{RwLock, Arc, atomic::AtomicBool}, collections::BTreeMap, };

use log::debug;
use near_base::{ObjectId, NearResult, Serialize, Deserialize, ErrorCode, NearError};
use near_transport::{ProcessTrait, RoutineEventTrait, Routine, ItfTrait, RoutineWrap, EventResult, process::EmptyTrait};
use near_util::{Topic, TopicRef};

use super::DispatchCallbackTrait;

struct SubMessageStateImpl {
    actived: AtomicBool,
}
type MessageStateMapRef = Arc<RwLock<BTreeMap<ObjectId, SubMessageStateImpl>>>;

struct SubMessageImpl {
    cb: Box<dyn DispatchCallbackTrait<DispatchMessage>>,
    messages: RwLock<BTreeMap<String, MessageStateMapRef>>,
}

#[derive(Clone)]
pub struct SubMessage(Arc<SubMessageImpl>);

impl SubMessage {
    pub fn new(cb: Box<dyn DispatchCallbackTrait<DispatchMessage>>) -> Self {
        Self(Arc::new(SubMessageImpl {
            cb,
            messages: RwLock::new(BTreeMap::new()),
        }))
    }
}

impl SubMessage {
    // pub(self) fn index_of(&self, from: &ObjectId) -> (Option<MessageStateMapRef>, Option<SubMessageStateImpl>) {

    // }

    pub(super) fn subscribe(&self, from: &ObjectId, secondary: &str) -> NearResult<()> {
        let message_state = {
            let messages = &mut *self.0.messages.write().unwrap();
            match messages.get(secondary) {
                Some(m) => 
                    m.clone(),
                None => {
                    let m = MessageStateMapRef::new(RwLock::new(BTreeMap::new()));
                    messages.insert(secondary.to_owned(), m.clone());
                    m
                }
            }
        };

        let w = &mut *message_state.write().unwrap();
        match w.get(from) {
            Some(item) => {
                item.actived.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            None => {
                w.insert(from.clone(), SubMessageStateImpl {
                    actived: AtomicBool::new(true),
                });
            }
        }

        Ok(())
    }

    pub(super) fn dissubscribe(&self, from: &ObjectId, secondary: &str) -> NearResult<()> {
        let messages = self.0.messages.read().unwrap()
            .get(secondary)
            .map(| m | {
                m.clone()
            })
            .ok_or_else(|| {
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Cloud not {} topic in secondarys.", secondary))
            })?;

        messages.read().unwrap()
            .get(from)
            .map(| item | {
                item.actived.store(false, std::sync::atomic::Ordering::SeqCst);
            });

        Ok(())
    }

    pub fn get_target_array(&self, topic: &TopicRef) -> Option<Vec<ObjectId>> {
        let r = 
        self.0.messages.read().unwrap()
            .get(topic.secondary().unwrap_or_default())
            .map(| map | {
                map.read().unwrap().keys().cloned().collect()
            });
        r
    }
}

impl ProcessTrait for SubMessage {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, _sender: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {

        Ok(RoutineWrap::new(OnDispatchMessageRoutine::new(self.clone(),
                                                                   topic.topic().clone(), 
                                                                   self.0.cb.clone_as_dispatch())) as Box<dyn RoutineEventTrait>)

    }

}

pub struct DispatchMessage {
    text: Vec<u8>
}

impl std::fmt::Display for DispatchMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "text-size={}", self.text.len())
    }
}

impl Clone for DispatchMessage {
    fn clone(&self) -> Self {
        let len = self.text.len();
        let mut text = vec![0u8; len];

        unsafe {
            std::ptr::copy(self.text.as_ptr(), text.as_mut_ptr(), len);
        }

        Self{text}
    }
}

impl ItfTrait for DispatchMessage {}

impl Serialize for DispatchMessage {
    fn raw_capacity(&self) -> usize {
        self.text.len()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let len = self.text.len();
        buf.copy_from_slice(self.text.as_slice());

        Ok(&mut buf[len..])
    }
}

impl Deserialize for DispatchMessage {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let len = buf.len();
        let mut text = vec![0u8; len];
        text.copy_from_slice(buf);

        Ok((DispatchMessage {
            text
        }, &buf[len..]))
    }
}

struct OnDispatchMessageRoutine {
    sub_manager: SubMessage,
    topic: Topic,
    cb: Box<dyn DispatchCallbackTrait<DispatchMessage>>,
}

impl OnDispatchMessageRoutine {
    pub fn new(sub_manager: SubMessage,
               topic: Topic, 
               cb: Box<dyn DispatchCallbackTrait<DispatchMessage>>) -> Box<Self> {
        Box::new(Self{
            sub_manager,
            topic,
            cb,
        })
    }
}

#[async_trait::async_trait]
impl Routine<DispatchMessage, EmptyTrait> for OnDispatchMessageRoutine {
    async fn on_routine(&self, from: &ObjectId, req: DispatchMessage) -> EventResult<EmptyTrait> {

        if let Ok(topic_ref) = self.topic.topic_d() {
            if let Some(target) = self.sub_manager.get_target_array(&topic_ref) {
                for a in target {
                    debug!("Dispatch to {} with topic {} and data {}", a, self.topic, req);

                    let _ = self.cb.on_dispatch(from, a, self.topic.clone(), req.clone());
                }
            }
        }

        EventResult::Ingnore
    }

}
