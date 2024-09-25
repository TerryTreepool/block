
mod queue_message;
mod sub_message;

use near_base::{ObjectId, NearResult};
use near_transport::{ItfTrait, };
use near_util::Topic;
pub use queue_message::{Message as QueueMessage};
pub use sub_message::DispatchMessage;

pub trait DispatchCallbackTrait<B: ItfTrait>: Send + Sync {
    fn clone_as_dispatch(&self) -> Box<dyn DispatchCallbackTrait<B>>;
    fn on_dispatch(&self, from: &ObjectId, target:ObjectId, topic: Topic, body: B) -> NearResult<()>;
}
