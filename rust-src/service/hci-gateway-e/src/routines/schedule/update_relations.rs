
use common::RoutineTemplate;
use log::{trace, error, info};

use near_base::{NearResult, NearError, ErrorCode};
use near_base::builder_codec_macro::Empty;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::hci::schedule::schedule_relation_list_update::Schedule_relation_list_op;
use protos::hci::schedule::{Schedule_info, Schedule_relation_list_update, Schedule_mode};
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object};
use topic_util::topics::hci_schedule::NEAR_THING_SCHEDULE_ADD_PUB;
use topic_util::topics::hci_service::{NEAR_THING_SERVICE_SCHEDULE_ADD_PUB, NEAR_THING_SERVICE_SCHEDULE_REMOVE_PUB};
use topic_util::topics::hci_storage::NEAR_THING_STORAGE_SCHEDULE_RELATIONS_UPDATE_PUB;

use crate::process::Process;

pub struct UpdateScheduleRelationsRoutine {
    _process: Process,
}

impl UpdateScheduleRelationsRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(UpdateScheduleRelationsRoutine{
            _process: process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for UpdateScheduleRelationsRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("UpdateScheduleRelationsRoutine: header_meta={header_meta}");

        let r = try_decode_raw_object!(Schedule_relation_list_update, req, o, o, { header_meta.sequence() });

        let r: DataContent<Schedule_info> = match r {
            DataContent::Content(schedule_relations) => self.on_routine(header_meta, schedule_relations).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl UpdateScheduleRelationsRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, mut schedule_relations: Schedule_relation_list_update) -> NearResult<Schedule_info> {

        let op = schedule_relations.op();
        if let Schedule_relation_list_op::none = op {
            Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, format!("undefined schedule op")))
        } else {
            Ok(())
        }?;

        let schedule = 
            RoutineTemplate::<Schedule_info>::call_with_headermeta(
                header_meta, 
                NEAR_THING_STORAGE_SCHEDULE_RELATIONS_UPDATE_PUB.topic().clone(), 
                schedule_relations.clone()
            )
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        if Schedule_mode::None == schedule.mode() {
            return Ok(schedule);
        }

        match op {
            Schedule_relation_list_op::update => {
                RoutineTemplate::<Empty>::call_with_headermeta(
                    header_meta, 
                    NEAR_THING_SERVICE_SCHEDULE_ADD_PUB.topic().clone(),
                    (
                                schedule_relations.take_schedule_id(), 
                                schedule_relations.take_relations(),
                            )
                )
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;    
            }
            Schedule_relation_list_op::remove => {
                RoutineTemplate::<Empty>::call_with_headermeta(
                    header_meta, 
                    NEAR_THING_SERVICE_SCHEDULE_REMOVE_PUB.topic().clone(),
                    (
                                schedule_relations.take_schedule_id(), 
                                {
                                    let v: Vec<String> = 
                                    schedule_relations.take_relations()
                                        .take_thing_relation()
                                        .into_iter()
                                        .map(| mut relation | {
                                            relation.take_thing_id()
                                        })
                                        .collect();
                                    v
                                }
                            )
                )
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;    
            }
            _ => unreachable!()
        }

        RoutineTemplate::<Empty>::call_with_headermeta(
            header_meta, 
            NEAR_THING_SCHEDULE_ADD_PUB.topic().clone(), 
            schedule.clone()
        )
        .await
        .map_err(| e | {
            error!("{e}, sequence: {}", header_meta.sequence());
            e
        })?
        .await
        .map(| _ | {
            info!("Successfully sync schedule: {schedule}, sequence: {}", header_meta.sequence());

            schedule
        })
        .map_err(| e | {
            error!("{e}, sequence: {}", header_meta.sequence());
            e
        })


    }
}