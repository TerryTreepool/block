
use std::{sync::{Arc, }, 
    };

use near_base::{NearResult, ObjectId};
use near_transport::{ProcessTrait, RoutineEventTrait, Stack, };
use near_util::{Topic, TopicRef};

use super::{MessageTrait, 
            core::{CoreMessage, CoreMessageCallbackTrait}, 
            queue::{QueueMessage, DispatchMessage, DispatchCallbackTrait}
        };

struct MessageComponents {
    core_message: CoreMessage,

    message_queue: QueueMessage,


    // kernel_message: 
    // system_message:
    // near_message:

    // message_center: MessageCenter,
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
    stack: Stack,

    message_components: Option<MessageComponents>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn new(stack: Stack) -> Self {
        let manager = Manager(Arc::new(ManagerImpl {
            stack,
            message_components: None,
            // message_centers: RwLock::new(BTreeMap::new()),
        }));

        // init message component
        let components = MessageComponents {
            core_message: CoreMessage::new(manager.clone_as_coremessage()),

            message_queue: QueueMessage::new(manager.clone_as_dispatch()),
            // kernel_message: 
            // system_message:
            // near_message:
            // message_center: MessageCenter::new(manager.clone()),
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
    fn queue_message(&self) -> &QueueMessage {
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

    fn on_subscribe(&self, from: &ObjectId, topic: Topic) -> NearResult<()> {
        // TODO: push into message queue
        self.queue_message().subscribe(from, topic)
    }

    fn on_dissubscribe(&self, from: &ObjectId, topic: Topic) -> NearResult<()> {
        self.queue_message().dissubscribe(from, topic)
    }
}

impl Manager {
    // pub(super) fn subscribe(&self, from: &ObjectId, topic: Topic) -> NearResult<()> {
    //     let topic_ref = topic.topic_d()?;
    //     let message = self.create_message_center(&topic_ref);

    //     message.subscribe(from.clone(), topic_ref)
    // }

    // fn message_center_of(&self, topic: &TopicRef) -> Option<MessageCenter> {
    //     self.0.message_centers.read().unwrap()
    //         .get(topic.primary())
    //         .map(| message | message.clone())
    // }

    // fn create_message_center(&self, topic: &TopicRef) -> MessageCenter {
    //     let centers = &mut *self.0.message_centers.write().unwrap();
    //     match centers.get(topic.primary()) {
    //         Some(message) => message.clone(),
    //         None => {
    //             let message = MessageCenter::new(self.clone());
    //             centers.insert(topic.primary().to_string(), message.clone());
    //             message
    //         }
    //     }
    // }
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

impl DispatchCallbackTrait<DispatchMessage> for Manager {
    fn clone_as_dispatch(&self) -> Box<dyn DispatchCallbackTrait<DispatchMessage>> {
        Box::new(self.clone())
    }

    fn on_dispatch(&self, _from: &ObjectId, target:ObjectId, topic: Topic, body: DispatchMessage) -> NearResult<()> {
        self.0.stack.post_message(Some(target), topic, body, None)
    }
}
