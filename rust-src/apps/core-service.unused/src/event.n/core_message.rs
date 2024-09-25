
use base::{SubscribeMessage, SubscribeMessageRespone};
use near_base::{NearResult, ObjectId};
use near_transport::{topic::TopicRoutineTrait, RoutineEventTrait, Routine, EventResult, RoutineWrap};

use crate::service::ServiceStack;

struct OnCoreSubscribeMessage {
    stack: ServiceStack,
}

impl OnCoreSubscribeMessage {
    pub fn new(stack: ServiceStack) -> Self {
        Self {
            stack
        }
    }
}

impl TopicRoutineTrait for OnCoreSubscribeMessage {
    fn on_topic_routine(&self) -> NearResult<Box<dyn RoutineEventTrait>> {

        struct SubscribeRoutine {
            stack: ServiceStack,
        }
        
        #[async_trait::async_trait]
        impl Routine<SubscribeMessage, SubscribeMessageRespone> for SubscribeRoutine {
            async fn on_routine(&self, from: &ObjectId, req: SubscribeMessage) -> EventResult<SubscribeMessageRespone> {
                // let r = 
                // match self.nds_stack
                //           .task_manager()
                //           .upload(from, req.chunk.clone())
                //           .await {
                //     Ok(_) => {
                //         ResponseEvent{data: InterestMessageResponse {
                //             chunk: req.chunk,
                //             errno: None,
                //         }}
                //     },
                //     Err(err) => {
                //         ResponseEvent{data: InterestMessageResponse {
                //             chunk: req.chunk,
                //             errno: Some(err),
                //         }}
                //     }
                // };

                // EventResult::Response(r)
                unimplemented!()
            }

        }

        Ok(RoutineWrap::new(Box::new(SubscribeRoutine{ stack: self.stack.clone() })) as Box<dyn RoutineEventTrait>)
    }
}