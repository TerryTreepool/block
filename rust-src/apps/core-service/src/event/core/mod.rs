
mod core_message;

pub use core_message::CoreMessage;

use near_base::{ObjectId, NearResult};
use near_util::Topic;
use base::MessageType;

pub trait CoreMessageCallbackTrait: Sync + Send {
    fn clone_as_coremessage(&self) -> Box<dyn CoreMessageCallbackTrait>;
    fn on_subscribe(&self, from: &ObjectId, topic: Topic, mt: MessageType) -> NearResult<()>;
    fn on_dissubscribe(&self, from: &ObjectId, topic: Topic) -> NearResult<()>;
}
