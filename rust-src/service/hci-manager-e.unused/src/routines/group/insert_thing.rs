
use log::{error, trace};

use near_base::{NearResult, ErrorCode, };
use near_core::time_utils::native_now;
use near_transport::HeaderMeta;

use protos::thing_group::Thing_group_relation_info;
use dataagent_util::Transaction;

use crate::process::Process;

pub struct InsertThingRoutine{
    process: Process,
}

impl InsertThingRoutine {
    pub fn new(process: Process) -> InsertThingRoutine {
        InsertThingRoutine {
            process,
        }
    }

}

impl InsertThingRoutine {

    async fn on_relation_routine(&self,
                                 trans: &mut Transaction,
                                 header_meta: &HeaderMeta,
                                 mut new_relation: Thing_group_relation_info) -> NearResult<()> {

        trace!("new_relation: {}", new_relation);

        enum NextStep {
            New(Thing_group_relation_info),
            Update(Thing_group_relation_info),
        }

        let next = 
        match crate::public::group::get_group_relation(
                    self.process.db_helper(), 
                    new_relation.group_id(),
                    new_relation.thing_id())
                .await {
            Ok(mut relation) => {
                relation.set_thing_data_property_text(new_relation.take_thing_data_property_text());
                relation.set_update_time(native_now().format("%Y-%m-%d %H:%M:%S").to_string());
                Ok(NextStep::Update(relation))
            }
            Err(e) => {
                match e.errno() {
                    ErrorCode::NEAR_ERROR_NOTFOUND => {
                        let now = native_now().format("%Y-%m-%d %H:%M:%S").to_string();
                        Ok(NextStep::New(Thing_group_relation_info {
                            group_id: new_relation.take_group_id(), 
                            thing_id: new_relation.take_thing_id(), 
                            thing_data_property_text: new_relation.take_thing_data_property_text(), 
                            begin_time: now.clone(), 
                            update_time: now, 
                            ..Default::default()
                        }))
                    }
                    _ => {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        Err(e)
                    }
                }
            }
        }?;

        match next {
            NextStep::New(relation) => 
                trans.execute_with_param(crate::p::ADD_THING_GROUP_RELATION.0, &relation)
                    .await
                    .map_err(| e | {
                        error!("{e}, sequence: {}", header_meta.sequence());
                        e
                    }),
            NextStep::Update(relation) => 
                trans.execute_with_param(crate::p::UPDATE_THING_GROUP_RELATION.0, &relation)
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                }),                
        }
    }

    pub async fn on_routine(&self, 
                            trans: &mut Transaction, 
                            header_meta: &HeaderMeta, 
                            thing_relations: Vec<Thing_group_relation_info>) -> NearResult<()> {

        for thing_relation in thing_relations {
            self.on_relation_routine(trans,
                                     header_meta,
                                     crate::public::group::thing_data_property::encode(thing_relation))
                .await?;
        }

        Ok(())
    }
}