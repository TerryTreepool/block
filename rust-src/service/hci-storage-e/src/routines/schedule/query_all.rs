
use log::{trace, error};

use near_base::NearResult;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, hci::schedule::{Schedule_list, Schedule_info}};

use crate::process::Process;

pub struct QueryAllScheduleRoutine {
    process: Process,
}

impl QueryAllScheduleRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(Self{
            process,
        }))
    }

}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for QueryAllScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, _req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("query all group routine: header_meta={header_meta}.");

        let r: DataContent<Schedule_list> = self.on_routine(header_meta).await.into();

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryAllScheduleRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta) -> NearResult<Schedule_list> {

        Ok(
            Schedule_list {
                schedules: {
                    self.process
                        .schedule_storage()
                        .load()
                        .await
                        .map_err(| e | {
                            error!("{e}, sequence: {}", header_meta.sequence());
                            e
                        })?
                        .into_iter()
                        .map(| schedule | {
                            let schedule: Schedule_info = schedule.into();

                            Schedule_info {
                                schedule_id: schedule.schedule_id,
                                schedule_name: schedule.schedule_name,
                                schedule_img_idx: schedule.schedule_img_idx,
                                mode: schedule.mode,
                                status: schedule.status,
                                ..Default::default()
                            }
                        })
                        .collect()
                },
                ..Default::default()
            }
        )

    }
}
