
use log::{trace, error};

use near_base::NearResult;
use near_transport::{EventResult, HeaderMeta, Routine, RoutineWrap, RoutineEventTrait};

use base::raw_object::RawObjectGuard;
use protos::{DataContent, try_decode_raw_object, try_encode_raw_object, hci::thing::{Thing_query_all, Thing_info_list}};

use crate::process::Process;

pub struct QueryAllThingRoutine {
    process: Process
}

impl QueryAllThingRoutine {
    pub fn new(process: Process) -> Box<dyn RoutineEventTrait> {
        RoutineWrap::new(Box::new(Self{
            process,
        }))
    }
}

#[async_trait::async_trait]
impl Routine<RawObjectGuard, RawObjectGuard> for QueryAllThingRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, req: RawObjectGuard) -> EventResult<RawObjectGuard> {
        trace!("query device routine: header_meta: {header_meta}");

        let r = try_decode_raw_object!(Thing_query_all, req, o, { (o.take_brand_id(), o.take_product_id()) }, { header_meta.sequence() });

        let r: DataContent<Thing_info_list> = match r {
            DataContent::Content((brand_id, product_id)) => self.on_routine(header_meta, brand_id, product_id).await.into(),
            DataContent::Error(e) => DataContent::Error(e),
        };

        try_encode_raw_object!(r, { header_meta.sequence() })
    }
}

impl QueryAllThingRoutine {
    async fn on_routine(&self, header_meta: &HeaderMeta, brand_id: String, major_product_id: String) -> NearResult<Thing_info_list> {

        if !brand_id.is_empty() {
            self.process
                .brand_storage()
                .load_with_prefix(&brand_id)
                .await
                .map(| _ | ())
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })
        } else {
            Ok(())
        }?;

        if !major_product_id.is_empty() {
            self.process
                .product_storage()
                .load_with_prefix(&major_product_id)
                .await
                .map(| _ | ())
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })
        } else {
            Ok(())
        }?;

        let things: Vec<protos::hci::thing::Thing_info> = 
            self.process
                .thing_storage()
                .load()
                .await
                .map_err(| e | {
                    error!("{e}, sequence: {}", header_meta.sequence());
                    e
                })?
                .into_iter()
                .map(| thing | {
                    let (thing, _) = thing.split();
                    thing
                })
                .collect();

        let things = 
            if !brand_id.is_empty() {
                things.into_iter()
                    .filter(| thing | {
                        thing.brand_id() == brand_id
                    })
                    .collect()
            } else {
                things
            };

        let things = 
            if !major_product_id.is_empty() {
                things.into_iter()
                    .filter(| thing | {
                        thing.major_product_id() == &major_product_id
                    })
                    .collect()
            } else {
                things
            };

        Ok(Thing_info_list {
            things,
            ..Default::default()
        })

    }
}

#[allow(unused)]
mod test {


    #[test]
    pub fn test_thing() {
        use std::path::PathBuf;

        use storage::{sqlite_storage::SqliteStorage, StorageTrait};

        use crate::caches::thing::ThingItem;
        use crate::caches::schedule::ScheduleItem;

        async_std::task::block_on(async move {
            let helper = SqliteStorage::new(PathBuf::new().join("D:\\A\\Documents\\WeChat Files\\xiaohj0724\\FileStorage\\File\\2023-09\\hci-storage.db").as_path()).unwrap();

            let schedule: Box<dyn StorageTrait<ScheduleItem>> = helper.add_storage("schedule").await.unwrap();

            let thing: Box<dyn StorageTrait<ThingItem>> = helper.add_storage("thing").await.unwrap();

            // match schedule.load_with_prefix(prefix)

            match thing.load_with_prefix("5SeZ1QQnkVwNfN99XMazCJdMW3iHQB4dzLkrQyUk2JWz").await {
                Ok(things) => {
                    // println!("{}", things.len());
                    // for thing in things {
                        let (a, b) = things.split();

                        println!("{a}");
                        println!("{b}");
                    // }
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
        });
    }
}
