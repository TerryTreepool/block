
use log::{error, trace, info};

use near_base::{NearResult, ObjectId, NearError, file::FileObject, builder_codec_macro::Empty, };
use near_transport::{RoutineEventTrait, Routine, EventResult, ResponseEvent, RoutineWrap, HeaderMeta, };

use base::{raw_object::RawObjectGuard};
use common::TopicRoutineCbEventTrait;
use protos::DataContent;

use crate::nds_protocol::PieceMessage;

use super::{NdsStack, 
            DownloadSource, SingleDownloadSource, MultiDownloadSource,
            nds_protocol::{SyncFileMessage, 
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
        impl Routine<RawObjectGuard, RawObjectGuard> for UploadFileRoutine {
            async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
                trace!("UploadFileRoutine::on_routine: header_meta={header_meta}, req={req}.");

                let r: DataContent<SyncFileMessage> = match protos::RawObjectHelper::decode::<SyncFileMessage>(req) {
                    Ok(r) => r.into(),
                    Err(e) => {
                        let error_string = format!("failed decode message with err = {e}");
                        error!("{error_string}, sequence = {}", header_meta.sequence());
                        Err(e)
                    }
                }.into();

                let r: DataContent<Empty> = match r {
                    DataContent::Content(file) => {
                        let from = header_meta.creator.as_ref().unwrap_or(&header_meta.requestor);

                        self.nds_stack
                            .task_manager()
                            .download_file(file.file, 
                                            MultiDownloadSource::new()
                                                .add_source(SingleDownloadSource::from(DownloadSource::default().set_target(from.clone()))))
                            .await
                            .map(| _ | {
                                info!("successful download file");
                                Empty
                            })
                            .map_err(| e | {
                                error!("{e}, sequence = {}", header_meta.sequence());
                                e
                            })
                    }
                    DataContent::Error(e) => Err(e),
                }.into();

                match protos::RawObjectHelper::encode(r) {
                    Ok(o) => EventResult::Response(o.into()),
                    Err(e) => {
                        error!("{e}, sequence = {}", header_meta.sequence());
                        EventResult::Ignore
                    }
                }
            }

        }

        Ok(RoutineWrap::new(Box::new(UploadFileRoutine{ nds_stack: self.stack.clone() })))
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
        impl Routine<RawObjectGuard, RawObjectGuard> for InterestRoutine {
            async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
                trace!("InterestRoutine::on_routine: header_meta={header_meta}, req={req}.");

                let r = match protos::RawObjectHelper::decode::<InterestMessage>(req) {
                    Ok(r) => r,
                    Err(e) => {
                        let error_string = format!("failed decode message with err = {e}");
                        error!("{error_string}, sequence = {}", header_meta.sequence());
                        DataContent::Error(e)
                    }
                };

                let r: DataContent<InterestMessageResponse> = match r {
                    DataContent::Content(interest) => {
                        self.nds_stack
                            .task_manager()
                            .upload(header_meta.creator.as_ref().unwrap_or(&header_meta.requestor).clone(), &interest)
                            .await
                            .map_err(
                                | e | {
                                error!("Failed to interest-chunk(from={:?}, req.chunk={}) with err = {} on sequence = {}", 
                                            header_meta.creator, 
                                            interest.chunk, 
                                            e, 
                                            header_meta.sequence());
                                e
                            })
                            .map(| _ | {
                                InterestMessageResponse{
                                    chunk: interest.chunk,
                                }
                            })
                    }
                    DataContent::Error(e) => Err(e)
                }.into();

                match protos::RawObjectHelper::encode(r) {
                    Ok(o) => { EventResult::Response(o.into()) },
                    Err(e) => {
                        error!("{e}, sequence = {}", header_meta.sequence());
                        EventResult::Ignore
                    }
                }
            }

        }

        Ok(RoutineWrap::new(Box::new(InterestRoutine{ nds_stack: self.stack.clone() })))
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
        impl Routine<RawObjectGuard, RawObjectGuard> for SyncPieceRoutine {
            async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
                trace!("SyncPieceRoutine::on_routine: header_meta={header_meta}, req={req}.");

                let r = match protos::RawObjectHelper::decode::<PieceMessage>(req) {
                    Ok(data) => data,
                    Err(e) => {
                        let error_string = format!("failed decode message with err = {e}");
                        error!("{error_string}, sequence = {}", header_meta.sequence());
                        DataContent::Error(e)
                    }
                };

                let r: DataContent<Empty> = match r {
                    DataContent::Content(data) => {
                        self.nds_stack
                            .task_manager()
                            .on_piece_data(&data)
                            .await
                            .map(| _ | {
                                Empty
                            })
                            .map_err(| e | {
                                error!("Failed to sync-piece(from={:?}, chunk={}, desc={}) with err = {} on sequence = {}", 
                                        header_meta.creator,
                                        data.chunk,
                                        data.desc,
                                        e,
                                        header_meta.sequence());
                                e
                            })
                    }
                    DataContent::Error(e) => Err(e),
                }.into();

                match protos::RawObjectHelper::encode(r) {
                    Ok(o) => { EventResult::Response(o.into()) },
                    Err(e) => {
                        error!("{e}, sequence = {}", header_meta.sequence());
                        EventResult::Ignore
                    }
                }
            }

        }

        Ok(RoutineWrap::new(Box::new(SyncPieceRoutine{ nds_stack: self.stack.clone() })))
    }
}


