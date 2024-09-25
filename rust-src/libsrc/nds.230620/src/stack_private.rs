
use base::TopicRoutineCbEventTrait;
use log::{error, trace};
use near_base::{NearResult, ObjectId, };
use near_transport::{RoutineEventTrait, Routine, EventResult, ResponseEvent, RoutineWrap, };


use crate::nds_protocol::PieceMessage;

use super::{NdsStack, 
            DownloadSource, SingleDownloadSource, MultiDownloadSource,
            nds_protocol::{SyncFileMessage, SyncFileMessageResponse,
                           InterestMessage, InterestMessageResponse,
            },
    };

pub struct OnNdsSyncFile {
    stack: NdsStack,
}

impl OnNdsSyncFile {
    pub fn new(stack: NdsStack) -> OnNdsSyncFile {
        Self{
            stack
        }
    }
}

impl TopicRoutineCbEventTrait for OnNdsSyncFile {
    fn on_topic_routine(&self) -> NearResult<Box<dyn RoutineEventTrait>> {
        trace!("enter.");

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
                        EventResult::Ingnore
                        // EventResult::Response(ResponseEvent {
                        //     data: SyncFileMessageResponse { errno: NearError::default() }
                        // })
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
    pub fn new(stack: NdsStack) -> OnNdsInterest {
        Self{
            stack
        }
    }
}

impl TopicRoutineCbEventTrait for OnNdsInterest {
    fn on_topic_routine(&self) -> NearResult<Box<dyn RoutineEventTrait>> {
        trace!("enter.");

        struct InterestRoutine {
            nds_stack: NdsStack,
        }
        
        #[async_trait::async_trait]
        impl Routine<InterestMessage, InterestMessageResponse> for InterestRoutine {
            async fn on_routine(&self, from: &ObjectId, req: InterestMessage) -> EventResult<InterestMessageResponse> {
                match self.nds_stack
                          .task_manager()
                          .upload(from.clone(), &req)
                          .await {
                    Ok(_) => {
                        // ResponseEvent{data: InterestMessageResponse {
                        //     chunk: req.chunk,
                        //     errno: None,
                        // }}
                    },
                    Err(err) => {
                        error!("Failed to interest-chunk(from={}, req.chunk={}) with err = {}", from, req.chunk, err);
                        // ResponseEvent{data: InterestMessageResponse {
                        //     chunk: req.chunk,
                        //     errno: Some(err),
                        // }}
                    }
                };

                // EventResult::Response(r)
                EventResult::Ingnore
            }

        }

        Ok(RoutineWrap::new(Box::new(InterestRoutine{ nds_stack: self.stack.clone() })) as Box<dyn RoutineEventTrait>)
    }
}

pub struct OnNdsPieceData {
    stack: NdsStack,
}

impl OnNdsPieceData {
    pub fn new(stack: NdsStack) -> Self {
        Self{
            stack
        }
    }
}

impl TopicRoutineCbEventTrait for OnNdsPieceData {
    fn on_topic_routine(&self) -> NearResult<Box<dyn RoutineEventTrait>> {
        trace!("enter.");

        struct SyncPieceRoutine {
            nds_stack: NdsStack,
        }
        
        #[async_trait::async_trait]
        impl Routine<PieceMessage, EmptyTrait> for SyncPieceRoutine {
            async fn on_routine(&self, from: &ObjectId, req: PieceMessage) -> EventResult<EmptyTrait> {
                let _ = 
                match self.nds_stack
                          .task_manager()
                          .on_piece_data(&req)
                          .await {
                    Ok(_) => {},
                    Err(err) => {
                        error!("Failed to sync-piece(from={}, chunk={}, desc={}) with err = {}", from, req.chunk, req.desc, err);
                    }
                };

                EventResult::Ingnore
            }

        }

        Ok(RoutineWrap::new(Box::new(SyncPieceRoutine{ nds_stack: self.stack.clone() })) as Box<dyn RoutineEventTrait>)
    }
}


