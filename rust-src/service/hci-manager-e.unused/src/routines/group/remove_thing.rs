
use dataagent_util::Transaction;
use log::{trace, error};

use near_base::{NearResult, builder_codec_utils::Empty, };
use near_transport::{EventResult, HeaderMeta, Routine, RoutineEventTrait, RoutineWrap};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_encode_raw_object, thing_group::{Thing_group_relation_info, Thing_group_relation_list}, try_decode_raw_object};

use crate::process::Process;

pub struct RemoveThingRoutine {
    process: Process,
}

impl RemoveThingRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(RemoveThingRoutine{
            process
        }))
    }

    #[inline]
    pub(self) fn process(&self) -> &Process {
        &self.process
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for RemoveThingRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("update group routine: header_meta={header_meta}");

        let r = 
            try_decode_raw_object!(Thing_group_relation_list, req, o, { o.take_thing_relation() }, { header_meta.sequence() });

        let r: DataContent<Empty> = match r {
            DataContent::Content(relation) => self.on_routine(header_meta, relation).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl RemoveThingRoutine {
    async fn on_relation_routine(&self, trans: &mut Transaction, header_meta: &HeaderMeta, relation: &Thing_group_relation_info) -> NearResult<()> {
        trans.execute_with_param(
            crate::p::DELETE_THING_GROUP_RELATION.0, 
            relation
        )
        .await
        .map_err(| e | {
            error!("{e}, sequence: {}", header_meta.sequence());
            e
        })
    }

    async fn on_routine(&self, header_meta: &HeaderMeta, relation_list: Vec<Thing_group_relation_info>) -> NearResult<Empty> {

        let mut trans = 
            self.process().db_helper().begin_transaction()
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?;

        for relation in relation_list.iter() {
            self.on_relation_routine(&mut trans, header_meta, relation)
                .await?
        }

        trans.commit().await
            .map_err(| e | {
                error!("{e}, sequence: {}", header_meta.sequence());
                e
            })?;

        Ok(Empty)

    }
}