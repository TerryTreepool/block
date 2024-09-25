
pub mod queue_message;
pub mod sub_message;

use near_base::{ObjectId, NearResult};
use near_transport::RoutineEventTrait;
use near_util::Topic;

pub trait DispatchCallbackTrait: Send + Sync {
    fn clone_as_dispatch(&self) -> Box<dyn DispatchCallbackTrait>;
    fn on_dispatch(&self, from: &ObjectId, target:ObjectId, topic: Topic, body: Vec<u8>, callback: Box<dyn RoutineEventTrait>) -> NearResult<()>;
}
