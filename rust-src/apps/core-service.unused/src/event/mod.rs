
// mod events;
mod core;
mod queue;
// mod general;
// mod message_center;
mod manager;

// pub use events::*;
pub use manager::*;
use near_transport::{ProcessTrait};

pub trait MessageTrait: ProcessTrait + Send + Sync {
    fn clone_as_message(&self) -> Box<dyn MessageTrait>;
    fn is_core_message(&self) -> bool;
    fn primary_label(&self) -> Option<&str>;
}

// pub trait MessageRoutineTrait: Send + Sync {
//     fn create_routine(&self, command: &[&str]) -> NearResult<Box<dyn RoutineEventTrait>>;
// }
