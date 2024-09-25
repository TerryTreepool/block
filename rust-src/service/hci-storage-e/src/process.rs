
use std::{sync::Arc, path::PathBuf};

use log::trace;

use near_core::get_service_path;
use near_base::NearResult;

use common::{RuntimeProcessTrait, RuntimeStack};

use storage::StorageTrait;
use storage::sqlite_storage::SqliteStorage;
use topic_util::topics::hci_storage::*;

use crate::caches::brand::BrandItem;
use crate::caches::product::ProductItem;
use crate::caches::schedule::ScheduleItem;
use crate::caches::thing::ThingItem;
use crate::routines::brand::add_brand::AddBrandRoutine;
use crate::routines::brand::query_all_brand::QueryAllBrandRoutine;
use crate::routines::brand::query_brand::QueryBrandRoutine;
use crate::routines::brand::remove_brand::RemoveBrandRoutine;
use crate::routines::schedule::add::AddScheduleRoutine;
use crate::routines::schedule::insert_thing::InsertThingRelationRoutine;
use crate::routines::schedule::query::QueryScheduleRoutine;
use crate::routines::schedule::query_all::QueryAllScheduleRoutine;
use crate::routines::schedule::remove_thing::RemoveThingRelationRoutine;
use crate::routines::schedule::update::UpdateScheduleRoutine;
use crate::routines::schedule::remove::RemoveScheduleRoutine;
use crate::routines::schedule::update_relations::UpdateRelationsRoutine;
use crate::routines::thing::add::AddThingRoutine;
use crate::routines::thing::query_all::QueryAllThingRoutine;
use crate::routines::thing::query_thingobject::QueryThingObjectRoutine;
use crate::routines::thing::query_multiple_thingobject::QueryMultipleThingObjectRoutine;
use crate::routines::thing::remove::RemoveThingRoutine;
use crate::routines::thing::update::UpdateThingRoutine;
use crate::routines::product::remove::RemoveProductRoutine;
use crate::routines::product::add::AddProductRoutine;
use crate::routines::product::query::QueryProductRoutine;
use crate::routines::product::query_all::QueryAllProductRoutine;

#[derive(Clone)]
#[allow(unused)]
pub(crate) struct Config {
    pub(crate) work_path: PathBuf,
    pub(crate) thing_data_path: PathBuf,
}

struct ProcessComponents {
    brand_storage: Box<dyn StorageTrait<BrandItem>>,
    product_storage: Box<dyn StorageTrait<ProductItem>>,
    thing_storage: Box<dyn StorageTrait<ThingItem>>,
    schedule_storage: Box<dyn StorageTrait<ScheduleItem>>,
}

struct ProcessImpl {
    #[allow(unused)]
    service_name: String,
    #[allow(unused)]
    config: Config,
    storage: SqliteStorage,

    components: Option<ProcessComponents>,
}

#[derive(Clone)]
pub struct Process(Arc<ProcessImpl>);

impl Process {
    pub async fn new(service_name: &str) -> NearResult<Box<Self>> {
        let config = {
            let work_path = get_service_path(service_name);
            Config {
                thing_data_path: {
                    let thing_data_path = work_path.join("thing-datas");
                    let _ = std::fs::create_dir_all(thing_data_path.as_path());
                    thing_data_path
                },
                work_path: work_path,
            }    
        };

        let ret = Self(Arc::new(ProcessImpl{
            service_name: service_name.to_owned(),
            config: config.clone(),
            storage: SqliteStorage::new(config.work_path.join(format!("{service_name}.db")).as_path())?,
            components: None,
        }));

        let mut_ret = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut ProcessImpl) };
        mut_ret.components = Some(ProcessComponents {
            brand_storage: ret.0.storage.add_storage("brand").await?,
            product_storage: ret.0.storage.add_storage("product").await?,
            thing_storage: ret.0.storage.add_storage("thing").await?,
            schedule_storage: ret.0.storage.add_storage("schedule").await?,
        });

        Ok(Box::new(ret))
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn config(&self) -> &Config {
        &self.0.config
    }

    // #[inline]
    // pub(crate) fn db_helper(&self) -> &Helper {
    //     &self.0.helper
    // }
    #[inline]
    pub(crate) fn brand_storage(&self) -> &dyn StorageTrait<BrandItem> {
        self.0.components.as_ref().unwrap().brand_storage.as_ref()
    }

    #[inline]
    pub(crate) fn product_storage(&self) -> &dyn StorageTrait<ProductItem> {
        self.0.components.as_ref().unwrap().product_storage.as_ref()
    }

    #[inline]
    pub(crate) fn thing_storage(&self) -> &dyn StorageTrait<ThingItem> {
        self.0.components.as_ref().unwrap().thing_storage.as_ref()
    }

    #[inline]
    pub(crate) fn schedule_storage(&self) -> &dyn StorageTrait<ScheduleItem> {
        self.0.components.as_ref().unwrap().schedule_storage.as_ref()
    }
    // #[inline]
    // pub(crate) fn product_manager(&self) -> &Manager<ProductItem> {
    //     &self.0.components.as_ref().unwrap().product_manager
    // }

    // #[inline]
    // pub(crate) fn device_manager(&self) -> &Manager<DeviceItem> {
    //     &self.0.components.as_ref().unwrap().device_manager
    // }

    // #[inline]
    // pub(crate) fn group_manager(&self) -> &Manager<GroupItem> {
    //     &self.0.components.as_ref().unwrap().group_manager
    // }
}

impl Process {

    // pub(self) fn load_sqls(&self) -> NearResult<()> {
    //     trace!("load_sqls enter");

    //     let sqlmaps =
    //         vec![crate::p::CREATE_BRAND, crate::p::ADD_BRAND, crate::p::UPDATE_BRAND, crate::p::GET_ALL_BRAND, crate::p::GET_BRAND,
    //              crate::p::CREATE_PRODUCT, crate::p::GET_ALL_PRODUCT, crate::p::GET_PRODUCT, crate::p::ADD_PRODUCT, crate::p::UPDATE_PRODUCT,
    //              crate::p::CREATE_DEVICE, crate::p::GET_ALL_DEVICE, crate::p::ADD_DEVICE, crate::p::UPDATE_DEVICE,
    //              crate::p::CREATE_THING_GROUP, crate::p::ADD_GROUP, crate::p::UPDATE_GROUP, crate::p::GET_ALL_GROUP, crate::p::GET_GROUP,
    //              crate::p::CREATE_GROUP_RELATION, crate::p::ADD_THING_GROUP_RELATION, crate::p::UPDATE_THING_GROUP_RELATION, crate::p::QUERY_THING_GROUP_RELATION, crate::p::QUERY_ALL_THING_GROUP_RELATION, crate::p::DELETE_THING_GROUP_RELATION, ];

    //     for (id, input, output,  sql) in sqlmaps {
    //         self.db_helper().add_sql(id, input, output, sql.to_owned())?;
    //     }

    //     Ok(())
    // }

    // pub(self) async fn init_sqls(&self) -> NearResult<()> {
    //     trace!("init_sqls enter");

    //     self.load_sqls()?;

    //     // init db
    //     let db_helper = self.db_helper();

    //     let init_array = vec![crate::p::CREATE_BRAND,
    //                                                         crate::p::CREATE_PRODUCT,
    //                                                         crate::p::CREATE_DEVICE,
    //                                                         crate::p::CREATE_THING_GROUP,
    //                                                         crate::p::CREATE_GROUP_RELATION,];

    //     for (id, ..) in init_array {

    //         trace!("exceute {id}");

    //         db_helper.execute(id)
    //             .await
    //             .map_err(| err | {
    //                 error!("failed execute {id} with error {err}");
    //                 err
    //             })?;
    //     }

    //     info!("Finished loading sqls");

    //     Ok(())
    // }

    #[cfg(load_cache)]
    pub(self) async fn load_brand(&self) -> NearResult<()> {
        trace!("load brand enter.");

        // let brands = self.db_helper().query_all::<Brand_info>(crate::p::GET_ALL_BRAND.0).await?;

        // self.brand_manager().add_items(brands.into_iter());

        Ok(())
    }

    #[cfg(load_cache)]
    pub(self) async fn load_product(&self) -> NearResult<()> {
        trace!("load product enter.");

        // let products = self.db_helper().query_all::<Product_info>(crate::p::GET_ALL_PRODUCT.0).await?;

        // self.product_manager().add_items(products.into_iter());

        Ok(())
    }

    #[cfg(load_cache)]
    pub(self) async fn load_device(&self) -> NearResult<()> {
        trace!("load device enter.");

        // let devices = self.db_helper().query_all::<Device_info>(crate::p::GET_ALL_DEVICE.0).await?;

        // self.device_manager().add_items(devices.into_iter());

        Ok(())
    }

    #[cfg(load_cache)]
    pub(self) async fn load_group(&self) -> NearResult<()> {
        trace!("load group enter.");

        // let groups = self.db_helper().query_all::<Thing_group_info>(crate::p::GET_ALL_GROUP.0).await?;

        // self.group_manager().add_items(groups.into_iter());

        // let relations = self.db_helper().query_all::<thing_group_relation_info>(sql_id)

        Ok(())
    }

    pub(self) async fn loading_data(&self) -> NearResult<()> {
        trace!("load data enter");

        #[cfg(load_cache)]
        {
            self.load_brand().await?;
            self.load_product().await?;
            self.load_device().await?;
            self.load_group().await?;
        }

        Ok(())
    }
}

impl Process {
    // pub(self) async fn subscribe_pub_topic(&self) -> NearResult<()> {
    //     {
    //         // begin transaction
    //         let arc_self = self.clone();
    //         RuntimeStack::get_instance()
    //             .topic_routine_manager()
    //             .register_public_topic(NEAR_THING_STORAGE_PUB_BEGIN_PUB.topic(),
    //                                   move || {
    //                                     Ok(BeginRoutine::new(arc_self.clone()))
    //                                 })
    //             .await?;
    //     }

    //     {
    //         // commit transaction
    //         RuntimeStack::get_instance()
    //             .topic_routine_manager()
    //             .register_public_topic(NEAR_THING_STORAGE_PUB_COMMIT_PUB.topic(),
    //                                   move || {
    //                                     Ok(CommitRoutine::new())
    //                                 })
    //             .await?;
    //     }

    //     {
    //         // rollback transaction
    //         RuntimeStack::get_instance()
    //             .topic_routine_manager()
    //             .register_public_topic(NEAR_THING_STORAGE_PUB_ROLLBACK_PUB.topic(),
    //                                     move || {
    //                                     Ok(RollbackRoutine::new())
    //                                 })
    //             .await?;
    //     }

    //     Ok(())
    // }

    pub(self) async fn subscribe_brand_topic(&self) -> NearResult<()> {
        // query all brand
        {
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_BRAND_QUERY_ALL_PUB.topic(),
                    move || {
                        Ok(QueryAllBrandRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        // add brand
        {
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_BRAND_ADD_PUB.topic(),
                        move || {
                        Ok(AddBrandRoutine::new(arc_self.clone())) 
                    }
                )?;
        }

        {
            // query a brand
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_BRAND_QUERY_PUB.topic(),
                        move || {
                        Ok(QueryBrandRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // update brand
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_BRAND_REMOVE_PUB.topic(),
                        move || {
                        Ok(RemoveBrandRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        Ok(())
    }

    pub(self) async fn subscribe_product_topic(&self) -> NearResult<()> {
        {
            // add product
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_PRODUCT_ADD_PUB.topic(), 
                        move || {
                        Ok(AddProductRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // update product
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_PRODUCT_REMOVE_PUB.topic(), 
                        move || {
                        Ok(RemoveProductRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // query product
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_PRODUCT_QUERY_PUB.topic(), 
                        move || {
                        Ok(QueryProductRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // query all product
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_PRODUCT_QUERY_ALL_PUB.topic(), 
                        move || {
                        Ok(QueryAllProductRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        Ok(())
    }

    pub(self) async fn subscribe_thing_topic(&self) -> NearResult<()> {
        {
            // add device
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_THING_ADD_PUB.topic(), 
                        move || { 
                        Ok(AddThingRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // update device
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_THING_UPDATE_PUB.topic(), 
                        move || {
                        Ok(UpdateThingRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // remove thing
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_THING_REMOVE_PUB.topic(), 
                        move || {
                        Ok(RemoveThingRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // query all device
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_THING_QUERY_ALL_PUB.topic(), 
                        move || {
                        Ok(QueryAllThingRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // query thing object
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_THINGOBJECT_QUERY_PUB.topic(), 
                        move || {
                        Ok(QueryThingObjectRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // query multiple query thing object
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_MULITPLE_THINGOBJECT_QUERY_PUB.topic(), 
                        move || {
                        Ok(QueryMultipleThingObjectRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        Ok(())
    }

    pub(self) async fn subscribe_schedule_topic(&self) -> NearResult<()> {
        {
            // add schedule
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_SCHEDULE_ADD_PUB.topic(), 
                        move || { 
                        Ok(AddScheduleRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // update schedule
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_SCHEDULE_UPDATE_PUB.topic(), 
                        move || { 
                        Ok(UpdateScheduleRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // remove schedule
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_SCHEDULE_REMOVE_PUB.topic(), 
                        move || { 
                        Ok(RemoveScheduleRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // query schedule
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_SCHEDULE_QUERY_PUB.topic(), 
                        move || { 
                        Ok(QueryScheduleRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // query all schedule
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_SCHEDULE_QUERYALL_PUB.topic(), 
                        move || { 
                        Ok(QueryAllScheduleRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // insert schedule things
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_SCHEDULE_INSERT_THING_PUB.topic(), 
                        move || { 
                        Ok(InsertThingRelationRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // remove schedule things
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_SCHEDULE_REMOVE_THING_PUB.topic(), 
                        move || { 
                        Ok(RemoveThingRelationRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        {
            // update schedule relations
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_public_topic(
                    NEAR_THING_STORAGE_SCHEDULE_RELATIONS_UPDATE_PUB.topic(), 
                        move || { 
                        Ok(UpdateRelationsRoutine::new(arc_self.clone()))
                    }
                )?;
        }

        Ok(())
    }

    pub(self) async fn subscribe_topic(&self) -> NearResult<()> {
        // self.subscribe_pub_topic().await?;
        self.subscribe_brand_topic().await?;
        self.subscribe_product_topic().await?;
        self.subscribe_thing_topic().await?;
        self.subscribe_schedule_topic().await?;

        Ok(())
    }

}

// impl TopicRoutineOpEventTrait for Process {
//     fn subscribe_message(&self, topic: &near_util::Topic, expire: MessageExpire) -> NearResult<()> {
//         RuntimeStack::get_instance().subscribe_message(topic, expire)
//     }

//     fn dissubscribe_message(&self, topic: &near_util::Topic) -> NearResult<()> {
//         RuntimeStack::get_instance().dissubscribe_message(topic)
//     }
// }

#[async_trait::async_trait]
impl RuntimeProcessTrait for Process {

    async fn run(&self) -> NearResult<()> {
        trace!("run enter");

        // self.init_sqls().await?;
        self.loading_data().await?;

        self.subscribe_topic().await?;

        Ok(())
    }

    fn quit(&self) {
        trace!("quiting...");
    }

}
