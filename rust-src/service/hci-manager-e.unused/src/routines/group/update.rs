
use common::RoutineTemplate;
use log::{trace, error, info};

use near_base::{NearResult, ErrorCode, builder_codec_utils::Empty};
use near_core::time_utils::native_now;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object};
use protos::thing_group::Thing_group_info;
use protos::hci_schedule::{Schedule_data, Hci_schedule_add, hci_schedule_add::Schedule_type};
use topic_util::{types::brand_types::Status, topics::hci_schedule::NEAR_THING_SCHEDULE_ADD_PUB};

use crate::{process::Process, public::CheckTrait};

use super::insert_thing::InsertThingRoutine;

pub struct UpdateGroupRoutine {
    process: Process,
}

impl UpdateGroupRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(UpdateGroupRoutine{
            process
        }))
    }

    #[inline]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for UpdateGroupRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("update group routine: header_meta={header_meta}");

        let r = try_decode_raw_object!(Thing_group_info, req, o, o, { header_meta.sequence() });

        let r: DataContent<Thing_group_info> = match r {
            DataContent::Content(group) => self.on_routine(header_meta, group).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl UpdateGroupRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, mut group: Thing_group_info) -> NearResult<Thing_group_info> {

        let mut mut_group = 
        crate::public::group::get_group(self.process().db_helper(), group.group_id())
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;
    
        match mut_group.check_status() {
            Ok(_) => Ok(()),
            Err(e) => match e.errno() {
                ErrorCode::NEAR_ERROR_NO_AVAILABLE => Ok(()),
                _ => {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    Err(e)
                }
            }
        }?;

        if group.status > 0 {
            // check group status
            Status::try_from(group.status)
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

            mut_group.set_status(group.status);
        }
    
        mut_group.set_update_time(native_now().format("%Y-%m-%d %H:%M:%S").to_string());

        // begin
        let mut trans = 
            self.process().db_helper().begin_transaction().await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        // write group info to db
        trans.execute_with_param(crate::p::UPDATE_GROUP.0, &mut_group)
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        // write relation
        InsertThingRoutine::new(self.process.clone())
            .on_routine(&mut trans, 
                        header_meta, 
                        group.thing_relation().to_vec())
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        trans.commit().await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        mut_group.mut_thing_relation().append(group.mut_thing_relation());

        // sync to hci-schedule
        {
            super::p::sync_group_schedule(self.process().clone(), mut_group.group_id().to_owned());
        }
        

        Ok(mut_group)
    }
}