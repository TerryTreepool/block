
use near_base::{NearResult, ObjectId, NearError};
use near_transport::{RoutineEventTrait, Routine, EventResult, ResponseEvent, RoutineWrap};


use super::{NdsStack, 
            DownloadSource, SingleDownloadSource, MultiDownloadSource,
            topic_routine::TopicRoutineTrait,
            nds_protocol::{SyncFileMessage, SyncFileMessageResponse,
                           InterestMessage, InterestMessageResponse,
            },
    };

pub struct OnNdsSyncFile {
    stack: NdsStack,
}

impl OnNdsSyncFile {
    pub fn new(stack: NdsStack) -> Box<OnNdsSyncFile> {
        Box::new(Self{
            stack
        })
    }
}

impl TopicRoutineTrait for OnNdsSyncFile {
    fn on_topic_routine(&self) -> NearResult<Box<dyn RoutineEventTrait>> {
        struct UploadFileRoutine {
            nds_stack: NdsStack,
        }

        #[async_trait::async_trait]
        impl Routine<SyncFileMessage, SyncFileMessageResponse> for UploadFileRoutine {
            async fn on_routine(&self, from: &ObjectId, req: SyncFileMessage) -> EventResult<SyncFileMessageResponse> {

                match self.nds_stack
                          .task_manager()
                          .download_file(req.file, 
                                         MultiDownloadSource::new().add_source(SingleDownloadSource::from(DownloadSource::default().set_target(from.clone()))))
                          .await {
                    Ok(_) => {
                        EventResult::Response(ResponseEvent {
                            data: SyncFileMessageResponse { errno: NearError::default() }
                        })
                    }
                    Err(errno) => {
                        EventResult::Response(ResponseEvent {
                            data: SyncFileMessageResponse { errno }
                        })
                    }
                }
            }

        }

        Ok(RoutineWrap::new(Box::new(UploadFileRoutine{ nds_stack: self.stack.clone() })) as Box<dyn RoutineEventTrait>)
    }
}

pub struct OnNdsInterest {
    stack: NdsStack,
}

impl OnNdsInterest {
    pub fn new(stack: NdsStack) -> Box<OnNdsInterest> {
        Box::new(Self{
            stack
        })
    }
}

impl TopicRoutineTrait for OnNdsInterest {
    fn on_topic_routine(&self) -> NearResult<Box<dyn RoutineEventTrait>> {
        struct InterestRoutine {
            nds_stack: NdsStack,
        }
        
        #[async_trait::async_trait]
        impl Routine<InterestMessage, InterestMessageResponse> for InterestRoutine {
            async fn on_routine(&self, from: &ObjectId, req: InterestMessage) -> EventResult<InterestMessageResponse> {
                let r = 
                match self.nds_stack
                          .task_manager()
                          .upload(from, req.chunk.clone())
                          .await {
                    Ok(_) => {
                        ResponseEvent{data: InterestMessageResponse {
                            chunk: req.chunk,
                            errno: None,
                        }}
                    },
                    Err(err) => {
                        ResponseEvent{data: InterestMessageResponse {
                            chunk: req.chunk,
                            errno: Some(err),
                        }}
                    }
                };

                EventResult::Response(r)
            }

        }

        Ok(RoutineWrap::new(Box::new(InterestRoutine{ nds_stack: self.stack.clone() })) as Box<dyn RoutineEventTrait>)
    }
}

pub struct OnNdsPieceData {
    stack: NdsStack,
}

impl OnNdsPieceData {
    pub fn new(stack: NdsStack) -> Box<Self> {
        Box::new(Self{
            stack
        })
    }
}

impl TopicRoutineTrait for OnNdsPieceData {
    fn on_topic_routine(&self) -> NearResult<Box<dyn RoutineEventTrait>> {
        unimplemented!()
    }
}


