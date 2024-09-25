
use base::{SubscribeMessage, SubscribeMessageRespone};

use near_base::{ObjectId};
use near_transport::{Routine, EventResult, topic::Topic};

use super::{Manager, };

pub struct SubscribeMessageRoutine {
    manager: Manager,
}

impl SubscribeMessageRoutine {
    pub fn new(manager: Manager) -> Box<Self> {
        Box::new(SubscribeMessageRoutine{
            manager
        })
    }
}

#[async_trait::async_trait]
impl Routine<SubscribeMessage, SubscribeMessageRespone> for SubscribeMessageRoutine {
    async fn on_routine(&self, from: &ObjectId, req: SubscribeMessage) -> EventResult<SubscribeMessageRespone> {
        for message in req.message_list {
            self.manager.subscribe(from, Topic::from(message));
        }
        // req.message_list
        //     .
        //     .for_each(| message | {
        //         let topic = Topic::from(message)
        //         // if let Ok(topic) = Topic::try_from(message.as_str()) {
        //         //     let _ = self.manager.subscribe(from, topic);
        //         // }
        //     });

        unimplemented!()
        // EventResult::Response(ResponseEvent {
        //     data: SubscribeMessageRespone
        // })
    }

}
