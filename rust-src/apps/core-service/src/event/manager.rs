
use std::sync::Arc;

use base::MessageType;
use near_base::{NearResult, ObjectId};
use near_transport::{ProcessTrait, RoutineEventTrait, };
use near_util::{Topic, TopicRef};

use crate::process::Process;

use super::{MessageTrait, 
            core::{CoreMessage, CoreMessageCallbackTrait}, 
            queue::{queue_message::MessageQueue, DispatchCallbackTrait, }, 
        };

struct MessageComponents {
    core_message: CoreMessage,

    message_queue: MessageQueue,
}

impl MessageComponents {
    pub(super) fn message_of(&self, primary: &str) -> Box<dyn MessageTrait> {
        if self.core_message.primary_label().unwrap().eq_ignore_ascii_case(primary) {
            self.core_message.clone_as_message()
        } else {
            self.message_queue.clone_as_message()
        }
    }

    // pub(super) fn message_center(&self) -> MessageCenter {
    //     self.message_center.clone()
    // }
}

struct ManagerImpl {
    #[allow(unused)]
    stack: Process,

    message_components: Option<MessageComponents>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn new(stack: Process) -> Self {
        let manager = Manager(Arc::new(ManagerImpl {
            stack,
            message_components: None,
            // message_centers: RwLock::new(BTreeMap::new()),
        }));

        // init message component
        let components = MessageComponents {
            core_message: CoreMessage::new(manager.clone_as_coremessage()),
            message_queue: MessageQueue::new(manager.clone_as_dispatch()),
        };

        let manager_ptr = unsafe { &mut *(Arc::as_ptr(&manager.0) as *mut ManagerImpl) };
        manager_ptr.message_components = Some(components);

        manager
    }

    #[inline]
    fn components(&self) -> &MessageComponents {
        self.0.message_components.as_ref().unwrap()
    }

    #[inline]
    fn queue_message(&self) -> &MessageQueue {
        &self.0.message_components.as_ref().unwrap().message_queue
    }

    // fn message_center(&self) -> &MessageCenter {
    //     &self.components().message_center
    // }
    #[inline]
    pub(super) fn get_message_ptr(&self, primary_label: &str) -> Box<dyn MessageTrait> {
        self.components().message_of(primary_label)
    }

}

impl CoreMessageCallbackTrait for Manager {
    fn clone_as_coremessage(&self) -> Box<dyn CoreMessageCallbackTrait> {
        Box::new(self.clone())
    }

    fn on_subscribe(&self, from: &ObjectId, topic: Topic, mt: MessageType) -> NearResult<()> {
        // TODO: push into message queue
        self.queue_message().subscribe(from, topic, mt)
    }

    fn on_dissubscribe(&self, from: &ObjectId, topic: Topic) -> NearResult<()> {
        self.queue_message().dissubscribe(from, topic)
    }
}

impl ProcessTrait for Manager {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, from: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        self.get_message_ptr(topic.primary())
            .create_routine(from, topic)
    }
}

impl DispatchCallbackTrait for Manager {
    fn clone_as_dispatch(&self) -> Box<dyn DispatchCallbackTrait> {
        Box::new(self.clone())
    }

    fn on_dispatch(&self, _from: &ObjectId, _target:ObjectId, _topic: Topic, _body: Vec<u8>, _callback: Box<dyn RoutineEventTrait>) -> NearResult<()> {
        unimplemented!()
        // CoreStack::get_instance().stack().post_text_message(Some(target), topic, body, Some(callback))
    }
}
