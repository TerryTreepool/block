
use std::sync::Arc;

use near_base::{NearResult, ObjectId};
use near_transport::{ProcessTrait, RoutineEventTrait};

use super::{Manager, MessageTrait};

struct MessageImpl {
    manager: Manager,
    primary_label: String,
}

#[derive(Clone)]
pub struct Message(Arc<MessageImpl>);

impl Message {
    pub fn new(manager: Manager, primary_label: impl std::string::ToString) -> Self {
        Self(Arc::new(MessageImpl{
            manager,
            primary_label: primary_label.to_string(),
        }))
    }
}

impl MessageTrait for Message {
    fn clone_as_message(&self) -> Box<dyn MessageTrait> {
        Box::new(self.clone())
    }

    fn is_core_message(&self) -> bool {
        false
    }

    fn message_primary_label(&self) -> &str {
        self.0.primary_label.as_str()
    }
}

impl ProcessTrait for Message {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, from: &ObjectId, topic: &str) -> NearResult<Box<dyn RoutineEventTrait>> {
        unimplemented!()
    }
}
