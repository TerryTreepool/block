
use common::RoutineTemplate;
use dataagent_util::Transaction;
use log::{trace, error, info, debug};

use near_base::{ErrorCode, NearError, NearResult, builder_codec_utils::Empty, };
use near_core::time_utils::native_now;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object, thing_group::{Thing_group_add, Thing_group_info, Thing_group_relation_info}, hci_schedule::{Hci_schedule_add, hci_schedule_add::Schedule_type, Schedule_data}};
use topic_util::{types::brand_types::Status, topics::hci_schedule::NEAR_THING_SCHEDULE_ADD_PUB};

use crate::process::Process;
use super::{GroupIdBuilder, insert_thing::InsertThingRoutine};

pub struct AddGroupRoutine {
    process: Process,
}

impl AddGroupRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(AddGroupRoutine{
            process
        }))
    }

    #[inline]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for AddGroupRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("add group routine: header_meta={header_meta}");

        let r = 
            try_decode_raw_object!(Thing_group_add, req, o, { (o.take_group_name(), o.take_thing_relation()) }, { header_meta.sequence() });

        let r: DataContent<Thing_group_info> = match r {
            DataContent::Content((group_name, relation)) => 
                self.on_routine(header_meta, group_name, relation).await.into(),
            DataContent::Error(e) => DataContent::Error(e)
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl AddGroupRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, group_name: String, relation: Vec<Thing_group_relation_info>) -> NearResult<Thing_group_info> {
        let group_name = group_name.trim();
        if group_name.is_empty() {
            let error_string = format!("group name is empty.");
            error!("{error_string}, sequence: {}", header_meta.sequence());
            Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, error_string))
        } else {
            Ok(())
        }?;

        let native_now = native_now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut group = Thing_group_info {
            group_id:   GroupIdBuilder{
                            group_name: group_name,
                            _now: 0,
                        }.build(),
            group_name: group_name.to_owned(),
            thing_relation: relation.clone(),
            begin_time: native_now.clone(),
            update_time: native_now,
            status: Status::Eanbled.into(),
            ..Default::default()
        };

        let mut trans = 
            self.process().db_helper().begin_transaction().await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        {
            // update group
            self.update_group(&mut trans, header_meta, &group)
                .await?;
        }
        // write group info

        // write thing-relation
        InsertThingRoutine::new(self.process.clone())
            .on_routine(&mut trans,
                        header_meta, 
                        relation.clone())
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        trans.commit()
            .await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        group.set_thing_relation(relation);

        // sync to hci-schedule
        {
            super::p::sync_group_schedule(self.process().clone(), group.group_id().to_owned());
        }

        Ok(group)
    }

    async fn update_group(&self, trans: &mut Transaction, header_meta: &HeaderMeta, group: &Thing_group_info) -> NearResult<()> {

        enum NextStep<'a> {
            New(&'a Thing_group_info),
            Update(Thing_group_info),
        }

        let next_step = 
        match crate::public::group::get_group(self.process().db_helper(), group.group_id())
                .await {
            Ok(mut updating_group) => {
                debug!("get_group:{updating_group}, sequence: {}", header_meta.sequence());
                updating_group.set_status(group.status);
                updating_group.set_update_time(group.update_time().to_owned());
                Ok(NextStep::Update(updating_group))
            }
            Err(e) => {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_NOTFOUND => Ok(NextStep::New(group)),
                    _ => {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        Err(e)
                    }
                }
            }
        }?;

        match next_step {
            NextStep::New(group) => {
                trans.execute_with_param(crate::p::ADD_GROUP.0, 
                                        group)
                    .await
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })
            }
            NextStep::Update(updating_group) => {
                trans.execute_with_param(crate::p::UPDATE_GROUP.0, 
                                         &updating_group)
                    .await
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    })
            }
        }
    }
}