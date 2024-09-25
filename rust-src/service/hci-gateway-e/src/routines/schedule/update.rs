
use common::RoutineTemplate;
use log::{trace, error, info};

use near_base::NearResult;
use near_base::builder_codec_macro::Empty;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::hci::schedule::Schedule_info;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object};
use topic_util::topics::hci_schedule::{NEAR_THING_SCHEDULE_ADD_PUB, NEAR_THING_SCHEDULE_REMOVE_PUB};
use topic_util::topics::hci_storage::NEAR_THING_STORAGE_SCHEDULE_UPDATE_PUB;
use topic_util::types::Status;

use crate::process::Process;

pub struct UpdateScheduleRoutine {
    _process: Process,
}

impl UpdateScheduleRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(UpdateScheduleRoutine{
            _process: process
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for UpdateScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("UpdateScheduleRoutine: header_meta={header_meta}");

        let r = try_decode_raw_object!(Schedule_info, req, o, o, { header_meta.sequence() });

        let r: DataContent<Schedule_info> = match r {
            DataContent::Content(schedule) => self.on_routine(header_meta, schedule).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl UpdateScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, new_schedule: Schedule_info) -> NearResult<Schedule_info> {

        let schedule_info = 
            RoutineTemplate::<Schedule_info>::call_with_headermeta(
                header_meta,
                NEAR_THING_STORAGE_SCHEDULE_UPDATE_PUB.topic().clone(),
                new_schedule
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

        if let Ok(status) = Status::try_from(schedule_info.status) {
            match status {
                Status::Disabled => {
                    RoutineTemplate::<Empty>::call_with_headermeta(
                        header_meta, 
                        NEAR_THING_SCHEDULE_REMOVE_PUB.topic().clone(), 
                        schedule_info.schedule_id().to_owned()
                    )
                    .await
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })?
                    .await
                    .map(| _ | {
                        info!("Successfully remove schedule: {}, sequence: {}", schedule_info.schedule_id(), header_meta.sequence());

                        schedule_info
                    })
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })
                }
                Status::Eanbled => {
                    RoutineTemplate::<Empty>::call_with_headermeta(
                        header_meta, 
                        NEAR_THING_SCHEDULE_ADD_PUB.topic().clone(), 
                        schedule_info.clone()
                    )
                    .await
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })?
                    .await
                    .map(| _ | {
                        info!("Successfully sync schedule: {schedule_info}, sequence: {}", header_meta.sequence());

                        schedule_info
                    })
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })
                }
            }
        } else {
            unreachable!()
        }

    }
}