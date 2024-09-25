
use std::{sync::Arc, path::PathBuf};

use log::{trace, error, info, };

use near_core::get_service_path;
use near_base::{ObjectId, NearResult, };
use near_transport::{ProcessTrait, RoutineEventTrait, };
use near_util::TopicRef;

use base::MessageExpire;
use common::{RuntimeProcessTrait, RuntimeStack, TopicRoutineOpEventTrait};
use dataagent_util::Helper;

use topic_util::topics::hci_manager::{NEAR_THING_MANAGER_BRAND_QUERY_ALL_PUB, 
                                        NEAR_THING_MANAGER_BRAND_ADD_PUB, 
                                        NEAR_THING_MANAGER_BRAND_QUERY_PUB, 
                                        NEAR_THING_MANAGER_BRAND_UPDATE_PUB, 
                                        NEAR_THING_MANAGER_PRODUCT_ADD_PUB, 
                                        NEAR_THING_MANAGER_PRODUCT_UPDATE_PUB, 
                                        NEAR_THING_MANAGER_PRODUCT_QUERY_PUB, 
                                        NEAR_THING_MANAGER_PRODUCT_QUERY_ALL_PUB, 
                                        NEAR_THING_MANAGER_DEVICE_ADD_PUB, 
                                        NEAR_THING_MANAGER_DEVICE_UPDATE_PUB, 
                                        NEAR_THING_MANAGER_DEVICE_QUERY_PUB, 
                                        NEAR_THING_MANAGER_DEVICE_QUERY_ALL_PUB, 
                                        NEAR_THING_MANAGER_GROUP_ADD_PUB, 
                                        NEAR_THING_MANAGER_GROUP_UPDATE_PUB, 
                                        NEAR_THING_MANAGER_GROUP_QUERY_PUB, 
                                        NEAR_THING_MANAGER_GROUP_QUERYALL_PUB, 
                                        NEAR_THING_MANAGER_GROUP_REMOVE_THING_PUB,
                                        NEAR_THING_MANAGER_QUERY_MULITPLE_THINGOBJECT_PUB, 
                                        NEAR_THING_MANAGER_DEVICE_REMOVE_PUB, 
                                        NEAR_THING_MANAGER_PUB_BEGIN_PUB, 
                                        NEAR_THING_MANAGER_PUB_COMMIT_PUB, 
                                        NEAR_THING_MANAGER_PUB_ROLLBACK_PUB};

// use crate::manager::device::DeviceItem;
// use crate::manager::Manager;
// use crate::manager::brand::BrandItem;
// use crate::manager::group::GroupItem;
// use crate::manager::product::ProductItem;
use crate::routines::brand::add_brand::AddBrandRoutine;
use crate::routines::brand::query_all_brand::QueryAllBrandRoutine;
use crate::routines::brand::query_brand::QueryBrandRoutine;
use crate::routines::brand::update_brand::UpdateBrandRoutine;
use crate::routines::group::add::AddGroupRoutine;
use crate::routines::group::query::QueryGroupRoutine;
use crate::routines::group::query_all::QueryAllGroupRoutine;
use crate::routines::group::remove_thing::RemoveThingRoutine;
use crate::routines::group::update::UpdateGroupRoutine;
use crate::routines::public::BeginRoutine;
use crate::routines::public::CommitRoutine;
use crate::routines::public::RollbackRoutine;
use crate::routines::thing::add::AddDeviceRoutine;
use crate::routines::thing::query_thing::QueryThingRoutine;
use crate::routines::thing::query_all::QueryAllDeviceRoutine;
use crate::routines::thing::query_multiple_thing::QueryMultipleThingObjectRoutine;
use crate::routines::thing::remove::RemoveDeviceRoutine;
use crate::routines::thing::update::UpdateDeviceRoutine;
use crate::routines::product::update::UpdateProductRoutine;
use crate::routines::product::add_product::AddProductRoutine;
use crate::routines::product::query::QueryProductRoutine;
use crate::routines::product::query_all::QueryAllProductRoutine;

#[derive(Clone)]
pub(crate) struct Config {
    pub(crate) work_path: PathBuf,
    pub(crate) thing_data_path: PathBuf,
}

struct ProcessComponents {
    // brand_manager: Manager<BrandItem>,
    // product_manager: Manager<ProductItem>,
    // device_manager: Manager<DeviceItem>,
    // group_manager: Manager<GroupItem>,
}

struct ProcessImpl {
    #[allow(unused)]
    service_name: String,
    #[allow(unused)]
    config: Config,
    helper: Helper,

    components: Option<ProcessComponents>,
}

#[derive(Clone)]
pub struct Process(Arc<ProcessImpl>);

impl Process {
    pub fn new(service_name: &str) -> NearResult<Box<Self>> {
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
            helper: Helper::new(config.work_path.join(format!("{service_name}.db")).as_path(), None)?,
            components: None,
        }));

        let mut_ret = unsafe { &mut *(Arc::as_ptr(&ret.0) as *mut ProcessImpl) };
        mut_ret.components = Some(ProcessComponents {
            // brand_manager: Manager::new(),
            // product_manager: Manager::new(),
            // device_manager: Manager::new(),
            // group_manager: Manager::new(),
        });

        Ok(Box::new(ret))
    }

    #[inline]
    pub(crate) fn config(&self) -> &Config {
        &self.0.config
    }

    #[inline]
    pub(crate) fn db_helper(&self) -> &Helper {
        &self.0.helper
    }

    // #[inline]
    // pub(crate) fn brand_manager(&self) -> &Manager<BrandItem> {
    //     &self.0.components.as_ref().unwrap().brand_manager
    // }

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

    pub(self) fn load_sqls(&self) -> NearResult<()> {
        trace!("load_sqls enter");

        let sqlmaps =
            vec![crate::p::CREATE_BRAND, crate::p::ADD_BRAND, crate::p::UPDATE_BRAND, crate::p::GET_ALL_BRAND, crate::p::GET_BRAND,
                 crate::p::CREATE_PRODUCT, crate::p::GET_ALL_PRODUCT, crate::p::GET_PRODUCT, crate::p::ADD_PRODUCT, crate::p::UPDATE_PRODUCT,
                 crate::p::CREATE_DEVICE, crate::p::GET_ALL_DEVICE, crate::p::ADD_DEVICE, crate::p::UPDATE_DEVICE,
                 crate::p::CREATE_THING_GROUP, crate::p::ADD_GROUP, crate::p::UPDATE_GROUP, crate::p::GET_ALL_GROUP, crate::p::GET_GROUP,
                 crate::p::CREATE_GROUP_RELATION, crate::p::ADD_THING_GROUP_RELATION, crate::p::UPDATE_THING_GROUP_RELATION, crate::p::QUERY_THING_GROUP_RELATION, crate::p::QUERY_ALL_THING_GROUP_RELATION, crate::p::DELETE_THING_GROUP_RELATION, ];

        for (id, input, output,  sql) in sqlmaps {
            self.db_helper().add_sql(id, input, output, sql.to_owned())?;
        }

        Ok(())
    }

    pub(self) async fn init_sqls(&self) -> NearResult<()> {
        trace!("init_sqls enter");

        self.load_sqls()?;

        // init db
        let db_helper = self.db_helper();

        let init_array = vec![crate::p::CREATE_BRAND,
                                                            crate::p::CREATE_PRODUCT,
                                                            crate::p::CREATE_DEVICE,
                                                            crate::p::CREATE_THING_GROUP,
                                                            crate::p::CREATE_GROUP_RELATION,];

        for (id, ..) in init_array {

            trace!("exceute {id}");

            db_helper.execute(id)
                .await
                .map_err(| err | {
                    error!("failed execute {id} with error {err}");
                    err
                })?;
        }

        info!("Finished loading sqls");

        Ok(())
    }

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
    pub(self) async fn subscribe_pub_topic(&self) -> NearResult<()> {
        {
            // begin transaction
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_PUB_BEGIN_PUB.topic(),
                                      move || {
                                        Ok(BeginRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        {
            // commit transaction
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_PUB_COMMIT_PUB.topic(),
                                      move || {
                                        Ok(CommitRoutine::new())
                                    })
                .await?;
        }

        {
            // rollback transaction
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_PUB_ROLLBACK_PUB.topic(),
                                        move || {
                                        Ok(RollbackRoutine::new())
                                    })
                .await?;
        }

        Ok(())
    }

    pub(self) async fn subscribe_brand_topic(&self) -> NearResult<()> {
        // query all brand
        {
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_BRAND_QUERY_ALL_PUB.topic(),
                                      move || {
                                        Ok(QueryAllBrandRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        // add brand
        {
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_BRAND_ADD_PUB.topic(),
                                      move || {
                                        Ok(AddBrandRoutine::new(arc_self.clone())) 
                                    })
                .await?;
        }

        {
            // query a brand
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_BRAND_QUERY_PUB.topic(),
                                      move || {
                                        Ok(QueryBrandRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        {
            // update brand
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_BRAND_UPDATE_PUB.topic(),
                                      move || {
                                        Ok(UpdateBrandRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        Ok(())
    }

    pub(self) async fn subscribe_product_topic(&self) -> NearResult<()> {
        {
            // add product
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_PRODUCT_ADD_PUB.topic(), 
                                      move || {
                                        Ok(AddProductRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        {
            // update product
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_PRODUCT_UPDATE_PUB.topic(), 
                                      move || {
                                        Ok(UpdateProductRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        {
            // query product
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_PRODUCT_QUERY_PUB.topic(), 
                                      move || {
                                        Ok(QueryProductRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        {
            // query all product
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_PRODUCT_QUERY_ALL_PUB.topic(), 
                                      move || {
                                        Ok(QueryAllProductRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        Ok(())
    }

    pub(self) async fn subscribe_device_topic(&self) -> NearResult<()> {
        {
            // add device
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_DEVICE_ADD_PUB.topic(), 
                                      move || { 
                                        Ok(AddDeviceRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        {
            // update device
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_DEVICE_UPDATE_PUB.topic(), 
                                      move || {
                                        Ok(UpdateDeviceRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        {
            // remove device
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_DEVICE_REMOVE_PUB.topic(), 
                                      move || {
                                        Ok(RemoveDeviceRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        {
            // query device
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_DEVICE_QUERY_PUB.topic(), 
                                      move || {
                                        Ok(QueryThingRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        {
            // query all device
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_DEVICE_QUERY_ALL_PUB.topic(), 
                                      move || {
                                        Ok(QueryAllDeviceRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        {
            // query all thing object
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_QUERY_MULITPLE_THINGOBJECT_PUB.topic(), 
                                      move || {
                                        Ok(QueryMultipleThingObjectRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        Ok(())
    }

    pub(self) async fn subscribe_group_topic(&self) -> NearResult<()> {
        {
            // add group
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_GROUP_ADD_PUB.topic(), 
                                      move || { 
                                        Ok(AddGroupRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        {
            // update group
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_GROUP_UPDATE_PUB.topic(), 
                                      move || { 
                                        Ok(UpdateGroupRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        {
            // query group
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_GROUP_QUERY_PUB.topic(), 
                                      move || { 
                                        Ok(QueryGroupRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        {
            // query all group
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_GROUP_QUERYALL_PUB.topic(), 
                                      move || { 
                                        Ok(QueryAllGroupRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }

        {
            // insert thing
            let arc_self = self.clone();
            RuntimeStack::get_instance()
                .topic_routine_manager()
                .register_topic_event(NEAR_THING_MANAGER_GROUP_REMOVE_THING_PUB.topic(), 
                                      move || { 
                                        Ok(RemoveThingRoutine::new(arc_self.clone()))
                                    })
                .await?;
        }
        Ok(())
    }

    pub(self) async fn subscribe_topic(&self) -> NearResult<()> {
        self.subscribe_pub_topic().await?;
        self.subscribe_brand_topic().await?;
        self.subscribe_product_topic().await?;
        self.subscribe_device_topic().await?;
        self.subscribe_group_topic().await?;

        Ok(())
    }

}

impl TopicRoutineOpEventTrait for Process {
    fn subscribe_message(&self, topic: &near_util::Topic, expire: MessageExpire) -> NearResult<()> {
        RuntimeStack::get_instance().subscribe_message(topic, expire)
    }

    fn dissubscribe_message(&self, topic: &near_util::Topic) -> NearResult<()> {
        RuntimeStack::get_instance().dissubscribe_message(topic)
    }
}

#[async_trait::async_trait]
impl RuntimeProcessTrait for Process {

    async fn run(&self) -> NearResult<()> {
        trace!("run enter");

        self.init_sqls().await?;
        self.loading_data().await?;

        self.subscribe_topic().await?;

        Ok(())
    }

    fn quit(&self) {
        trace!("quiting...");
    }

}

impl ProcessTrait for Process {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, sender: &ObjectId, topic: &TopicRef) -> NearResult<Box<dyn RoutineEventTrait>> {
        trace!("from: {}, topic: {}, ", sender, topic);

        RuntimeStack::get_instance()
            .topic_routine_manager()
            .call(topic)
    }

}
