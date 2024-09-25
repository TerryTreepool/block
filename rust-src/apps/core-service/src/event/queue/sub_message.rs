
use std::{sync::{RwLock, Arc, }, 
          collections::{BTreeMap, btree_map::Entry, BTreeSet, }, 
    };

use base::MessageType;
use log::{error, trace};
use near_base::{ObjectId, NearResult, ErrorCode, NearError, ObjectTypeCode};
use near_transport::{ProcessTrait, RoutineEventTrait, EventResult, HeaderMeta, process::{provider::EventTextResult, TransferEvent}};
use near_util::{Topic, TopicRef};

use super::DispatchCallbackTrait;

struct MessageState {
    targets: RwLock<BTreeSet<Arc<ObjectId>>>,
    mt: MessageType,
}

impl MessageState {
    fn new(mt: MessageType) -> Self {
        Self {
            targets: RwLock::new(BTreeSet::new()),
            mt,
        }
    }
}

type MessageStateRef = Arc<MessageState>;

struct SubMessageImpl {
    cb: Box<dyn DispatchCallbackTrait>,
    messages: RwLock<BTreeMap<String, MessageStateRef>>,
}

#[derive(Clone)]
pub struct SubMessage(Arc<SubMessageImpl>);

impl SubMessage {
    pub fn new(cb: Box<dyn DispatchCallbackTrait>) -> Self {
        Self(Arc::new(SubMessageImpl {
            cb,
            messages: RwLock::new(BTreeMap::new()),
        }))
    }
}

impl SubMessage {

    pub(super) fn subscribe(&self, from: &ObjectId, topic: Topic, mt: MessageType) -> NearResult<()> {
        let message_state = {
            match self.0
                      .messages
                      .write().unwrap()
                      .entry(topic.into()) {
                Entry::Vacant(empty) => {
                    let m = MessageStateRef::new(MessageState::new(mt));
                    empty.insert(m.clone());
                    m
                }
                Entry::Occupied(exist) => {
                    exist.get().clone()
                }
            }
        };

        let w = &mut *message_state.targets.write().unwrap();
        match w.get(from) {
            Some(_) => {}
            None => {
                w.insert(Arc::new(from.clone()), );
            }
        }

        Ok(())
    }

    pub(super) fn dissubscribe(&self, from: &ObjectId, topic: Topic) -> NearResult<()> {
        let message_state = 
            self.0.messages.read().unwrap()
                .get(topic.topic())
                .cloned()
                .ok_or_else(|| {
                    let error_string = format!("Cloud not {} topic.", topic.topic());
                    error!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
                })?;

        message_state.targets
                .write().unwrap()
                .remove(from);

        Ok(())
    }

    pub fn get_target(&self, requestor: &ObjectId, topic: &Topic) -> NearResult<Vec<ObjectId>> {
        match {
            self.0.messages.read().unwrap()
                .get(topic.topic())
                .cloned()
        } {
            Some(state) => {
                match state.mt {
                    MessageType::Public => Ok(()),
                    MessageType::Private => {
                        if let ObjectTypeCode::People = requestor.object_type_code()? {
                            Err(NearError::new(ErrorCode::NEAR_ERROR_FORBIDDEN, format!("{topic} is forbbiden topic.")))
                        } else {
                            Ok(())
                        }
                    }
                }?;

                let r = 
                    state.targets
                        .read().unwrap()
                        .iter()
                        .map(| it | it.as_ref().clone())
                        .collect();

                Ok(r)
            }
            None => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("missing [{}] provider", topic)))
            }
        }

    }
}

impl ProcessTrait for SubMessage {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, _sender: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {

        Ok(OnDispatchMessageRoutine::new(self.clone(),
                                         topic.topic().clone(), 
                                         self.0.cb.clone_as_dispatch()))

    }

}

struct OnDispatchMessageRoutine {
    sub_manager: SubMessage,
    _topic: Topic,
    _cb: Box<dyn DispatchCallbackTrait>,
}

impl OnDispatchMessageRoutine {
    pub fn new(sub_manager: SubMessage,
               topic: Topic, 
               cb: Box<dyn DispatchCallbackTrait>) -> Box<Self> {
        Box::new(Self{
            sub_manager,
            _topic: topic,
            _cb: cb,
        })
    }
}

#[async_trait::async_trait]
impl RoutineEventTrait for OnDispatchMessageRoutine {
    async fn emit(&self, header_meta: &HeaderMeta, data: Vec<u8>) -> NearResult<EventTextResult> {

        trace!("OnDispatchMessageRoutine::emit sequence={}", header_meta.command.sequence());

        struct CallbackRoutine {
            requestor: ObjectId,
            topic: Topic,
        }

        #[async_trait::async_trait]
        impl RoutineEventTrait for CallbackRoutine {
            async fn emit(&self, header_meta: &HeaderMeta, data: Vec<u8>) -> NearResult<EventTextResult> {

                trace!(
                    "OnDispatchMessageRoutine::CallbackRoutine, target: {} sequence: {}, data-size: {}", 
                    &self.requestor, 
                    header_meta.sequence(), 
                    data.len()
                );

                Ok(EventTextResult::Transfer(TransferEvent{
                    to: vec![(self.requestor.clone(), None)],
                    topic: self.topic.clone().into(),
                    data,
                }))

            }
        }

        match self.sub_manager.get_target(&header_meta.requestor, &header_meta.topic) {
            Ok(targets) => {

                let to = {
                    let mut arrays = vec![];
                    for target in targets {
                        arrays.push((target, 
                                    Some(Box::new(CallbackRoutine {
                                        requestor: header_meta.requestor.clone(),
                                        topic: header_meta.topic.clone().into(),
                                    }) as Box<dyn RoutineEventTrait>)));
                    }
                    arrays
                };

                Ok(EventResult::Transfer(TransferEvent { to, topic: header_meta.topic.clone().into(), data }))
            }
            Err(e) => {
                error!("{e}, sequence: {}", header_meta.sequence());
                if let Ok(e) = protos::RawObjectHelper::encode_with_error(e) {
                    let r = EventResult::Response(e.into());

                    if let EventResult::Response(r) = r {
                        if let Ok(r) = EventTextResult::try_from(r) {
                            Ok(r)
                        } else {
                            Ok(EventResult::Ignore)
                        }
                    } else {
                        unreachable!()
                    }
                } else {
                    Ok(EventResult::Ignore)
                }
            }
        }

    }
}

