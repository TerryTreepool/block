
use std::{sync::{Arc, RwLock, }, collections::HashMap};

use log::{error, trace};
use once_cell::sync::OnceCell;

use near_base::{people::PeopleObject, PrivateKey, DeviceObject, NearResult, NearError, ErrorCode, Deserialize, Serialize, builder_codec_macro::Empty};
use near_transport::{process::ProcessEventTrait, ProcessTrait, RequestorMeta, RoutineWrap, Stack, StackOpenParams, StackPeopleParams };
use near_util::Topic;

use protobuf::Enum;
use protos::hci::product::*;
use protos::hci::schedule::{*, schedule_relation_list_update::Schedule_relation_list_op};
use protos::hci::thing::*;
use protos::hci::hci_thing::{*, hci_crud_thing::Hci_crud_m};
use protos::hci::brand::*;
use protos::DataContent;
use topic_util::types::hci_types::HciTaskId;

use crate::{cb::Cb, RequestCommon};

pub static CLI_STACK: OnceCell<CliStack> = OnceCell::new();

lazy_static::lazy_static! {
    static ref LAST_ERROR: RwLock<NearError> = RwLock::new(Default::default());
}

pub fn clr_last_error() {
    let _ = std::mem::replace(&mut *LAST_ERROR.write().unwrap(), Default::default());
}

pub fn set_last_error(error: NearError) {
    let mut_error = &mut *LAST_ERROR.write().unwrap();
    *mut_error = error;
}

pub fn get_last_error() -> NearError {
    LAST_ERROR.read().unwrap().clone()
}

pub struct CliStackBuild {
    pub(crate) people: PeopleObject,
    pub(crate) people_private_key: PrivateKey,
    pub(crate) core: DeviceObject,
}

impl CliStackBuild {
    pub(crate) async fn build(self) -> NearResult<()> {
        let cli_stack = CliStack::open(self.people, self.people_private_key, self.core).await?;

        CLI_STACK.set(cli_stack)
            .map_err(| _ | {
                NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN, "failed init cli-stack")
            })?;

        Ok(())
    }
}


struct CliStackImpl {
    stack: Option<Stack>,
}

#[derive(Clone)]
pub struct CliStack(Arc<CliStackImpl>);

impl CliStack {
    pub fn get_instance() -> Option<&'static CliStack> {
        CLI_STACK.get()
    }

    async fn open(people: PeopleObject, people_private_key: PrivateKey, core: DeviceObject) -> NearResult<Self> {
        let ret = CliStack(Arc::new(CliStackImpl {
                                stack: None
                            }));

        let stack = 
            Stack::open_people(StackPeopleParams {
                                    core_service: core,
                                    people,
                                    people_private_key,
                                    people_event_impl: ret.clone_as_process(),
                                    people_process_event_impl: Some(Box::new(ret.clone()) as Box<dyn ProcessEventTrait>),
                                }, 
                                StackOpenParams {
                                    config: None,
                                    device_cacher: None,
                                }).await?;

        let mut_ret = unsafe {
            &mut *(Arc::as_ptr(&ret.0) as *mut CliStackImpl)
        };
        mut_ret.stack = Some(stack);

        Ok(ret)
    }

    #[inline]
    pub(crate) fn stack(&self) -> &Stack {
        self.0.stack.as_ref().unwrap()
    }

    pub async fn wait_online(&self) -> bool {
        self.stack()
            .wait_online()
            .await
    }
}

impl CliStack {
    pub async fn query_all_brand(
        &self, 
        reqeust: RequestCommon, 
    ) -> NearResult<Brand_info_list> {
        let query_resp: Brand_info_list = 
            self.request_and_wait(
                reqeust,
                topic_util::topics::hci_storage::NEAR_THING_STORAGE_BRAND_QUERY_ALL_PUB.topic().clone(),
                Empty
            )
            .await?;

        Ok(query_resp)
    }

    pub async fn query_brand(
        &self, 
        reqeust: RequestCommon, 
        brand_id: String
    ) -> NearResult<Brand_info> {
        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_storage::NEAR_THING_STORAGE_BRAND_QUERY_PUB.topic().clone(),
            brand_id
        )
        .await
    }

    pub async fn add_brand(
        &self, 
        reqeust: RequestCommon, 
        new_brand: Brand_info
    ) -> NearResult<Brand_info> {
        let add_new_brand = Brand_add {
            brand: Some(new_brand).into(),
            ..Default::default()
        };

        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_storage::NEAR_THING_STORAGE_BRAND_ADD_PUB.topic().clone(),
            add_new_brand
        )
        .await
    }

    pub async fn remove_brand(
        &self, 
        reqeust: RequestCommon, 
        brand_id: String
    ) -> NearResult<Empty> {
        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_storage::NEAR_THING_STORAGE_BRAND_REMOVE_PUB.topic().clone(),
            brand_id
        )
        .await
    }

    pub async fn add_product(
        &self, 
        reqeust: RequestCommon, 
        major_product_id: Option<String>, 
        product_name: String
    ) -> NearResult<Product_info> {

        let product = Product_info {
            parent_product_id: major_product_id.unwrap_or_default(),
            product_name,
            ..Default::default()
        };

        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_storage::NEAR_THING_STORAGE_PRODUCT_ADD_PUB.topic().clone(),
            Product_add {
                product: Some(product).into(),
                ..Default::default()
            }
        )
        .await
    }

    pub async fn remove_product(
        &self, 
        reqeust: RequestCommon, 
        major_product_id: Option<String>,
        product_id: String
    ) -> NearResult<Empty> {
            
        let product = Product_info {
            parent_product_id: major_product_id.unwrap_or_default(),
            product_id,
            ..Default::default()
        };

        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_storage::NEAR_THING_STORAGE_PRODUCT_REMOVE_PUB.topic().clone(),
            product
        )
        .await
    }

    pub async fn query_product(
        &self, 
        reqeust: RequestCommon, 
        major_product_id: String
    ) -> NearResult<Product_info> {
        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_storage::NEAR_THING_STORAGE_PRODUCT_QUERY_PUB.topic().clone(),
            major_product_id
        )
        .await
    }

    pub async fn query_all_product(
        &self,
        reqeust: RequestCommon, 
    ) -> NearResult<Product_info_list> {
        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_storage::NEAR_THING_STORAGE_PRODUCT_QUERY_ALL_PUB.topic().clone(),
            Empty
        )
        .await
    }

    pub async fn update_thing(
        &self, 
        reqeust: RequestCommon, 
        thing_id: String, 
        thing_name: String
    ) -> NearResult<Thing_info> {

        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_storage::NEAR_THING_STORAGE_THING_UPDATE_PUB.topic().clone(),
            Thing_info {
                thing_id,
                thing_name,
                ..Default::default()
            }
        )
        .await
    }

    // pub async fn query_device(&self, device_id: String) -> NearResult<Device_info> {
    //     self.request_and_wait(topic_util::topics::hci_manager::NEAR_THING_MANAGER_DEVICE_QUERY_PUB.topic().clone(),
    //                           Device_query {
    //                             device_id,
    //                             ..Default::default()
    //                           })
    //         .await
    // }

    pub async fn query_all_thing(
        &self, 
        reqeust: RequestCommon, 
        brand_id: Option<String>, 
        product_id: Option<String>
    ) -> NearResult<Thing_info_list> {
        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_storage::NEAR_THING_STORAGE_THING_QUERY_ALL_PUB.topic().clone(),
            Thing_query_all {
            brand_id: brand_id.unwrap_or_default(),
            product_id: product_id.unwrap_or_default(),
            ..Default::default()
            }
        )
        .await
    }

    // schedule
    pub async fn add_schedule(
        &self, 
        reqeust: RequestCommon, 
        schedule_name: String,
        schedule_img_index: Option<u32>,
        schedule_mode: i32,
        relations: Vec<(String, HashMap<String, String>)>
    ) -> NearResult<Schedule_info> {

        let mode = 
            Schedule_mode::from_i32(schedule_mode)
                .ok_or_else(|| {
                    let error_string = format!("Undefined [{}] schedule-mode.", schedule_mode);
                    error!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, error_string)
                })?;

        let relations = 
            relations.into_iter()
                .map(| (thing_id, thing_data) | {
                    Schedule_relation_info {
                        thing_id,
                        thing_data_property: thing_data,
                        ..Default::default()
                    }
                })
                .collect();

        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_gateway::NEAR_THING_GATEWAY_SCHEDULE_ADD_PUB.topic().clone(),
            Schedule_add {
                schedule_name,
                thing_relation: relations,
                schedule_img_idx: schedule_img_index.unwrap_or(0),
                mode: mode.into(),
                ..Default::default()
            }
        )
        .await
    }

    pub async fn update_schedule_info(
        &self, 
        reqeust: RequestCommon, 
        schedule_id: String,
        schedule_name: Option<String>,
        schedule_img_index: Option<u32>,
        schedule_status: Option<u32>
    ) -> NearResult<Schedule_info> {
        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_gateway::NEAR_THING_GATEWAY_SCHEDULE_UPDATE_PUB.topic().clone(),
            Schedule_info {
                schedule_id,
                schedule_name: schedule_name.unwrap_or_default(),
                schedule_img_idx: schedule_img_index.unwrap_or(0),
                status: schedule_status.unwrap_or(0),
                ..Default::default()
            }
        )
        .await
    }

    pub async fn update_schedule_property(
        &self, 
        reqeust: RequestCommon, 
        schedule_id: String, 
        relations: Vec<(String, Vec<(String, String)>)>
    ) -> NearResult<Schedule_info> {

        let relations: Vec<Schedule_relation_info> = 
            relations.into_iter()
                .map(| (thing_id, thing_data) | {
                    Schedule_relation_info {
                        thing_id,
                        thing_data_property: thing_data.into_iter().collect(),
                        ..Default::default()
                    }
                })
                .collect();

        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_gateway::NEAR_THING_GATEWAY_SCHEDULE_UPDATE_RELATIONS_PUB.topic().clone(),
            Schedule_relation_list_update {
                schedule_id,
                op: Schedule_relation_list_op::update.into(),
                relations: Some(Schedule_relation_list{
                    thing_relation: relations,
                    ..Default::default()
                }).into(),
                ..Default::default()
            }
        )
        .await
    }

    pub async fn remove_schedule_property(
        &self, 
        reqeust: RequestCommon, 
        schedule_id: String, 
        relations: Vec<String>
    ) -> NearResult<Schedule_info> {

        let relations = 
            relations.into_iter()
                .map(| thing_id | {
                    Schedule_relation_info {
                        thing_id,
                        thing_data_property: Default::default(),
                        ..Default::default()
                    }
                })
                .collect();

        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_gateway::NEAR_THING_GATEWAY_SCHEDULE_UPDATE_RELATIONS_PUB.topic().clone(),
            Schedule_relation_list_update {
                schedule_id,
                op: Schedule_relation_list_op::remove.into(),
                relations: Some(Schedule_relation_list{
                    thing_relation: relations,
                    ..Default::default()
                }).into(),
                ..Default::default()
            }
        )
        .await
    }

    pub async fn update_timeperiod_schedule_info(
        &self,
        reqeust: RequestCommon, 
        schedule_id: String,
        hour: u32, minute: u32,
        cycle_week: u32,
    ) -> NearResult<Schedule_info> {

        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_gateway::NEAR_THING_GATEWAY_SCHEDULE_UPDATE_PUB.topic().clone(),
            Schedule_info {
                schedule_id,
                timeperiod_mode: Some(Schedule_timeperiod_mode {
                    time: Some(Schedule_cycle_time {
                        hour, minute,
                        ..Default::default()
                    }).into(),
                    cycle: cycle_week,
                    ..Default::default()
                }).into(),
                ..Default::default()
            }
        )
        .await
    
    }

    pub async fn remove_schedule(
        &self, 
        reqeust: RequestCommon, 
        schedule_id: String
    ) -> NearResult<Schedule_info> {
        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_gateway::NEAR_THING_GATEWAY_SCHEDULE_REMOVE_PUB.topic().clone(),
            schedule_id
        )
        .await

    }

    pub async fn query_schedule(
        &self, 
        reqeust: RequestCommon, 
        schedule_id: String
    ) -> NearResult<Schedule_info> {

        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_storage::NEAR_THING_STORAGE_SCHEDULE_QUERY_PUB.topic().clone(),
            schedule_id
        )
        .await
    }

    pub async fn query_all_simple_schedule(
        &self,
        reqeust: RequestCommon, 
    ) -> NearResult<Schedule_list> {

        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_storage::NEAR_THING_STORAGE_SCHEDULE_QUERYALL_PUB.topic().clone(),
            Empty
        )
        .await
    }

    pub async fn execute_schedule(
        &self, 
        reqeust: RequestCommon, 
        schedule_id: String
    ) -> NearResult<Empty> {
        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_schedule::NEAR_THING_SCHEDULE_EXECUTE_PUB.topic().clone(),
            schedule_id
        )
        .await
    }

    pub async fn hci_search_thing(
        &self, 
        reqeust: RequestCommon, 
        brand_id: String
    ) -> NearResult<HciTaskId> {
        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_gateway::NEAR_THING_GATEWAY_SEARCH_PUB.topic().clone(), 
            brand_id
        )
        .await
    }

    pub async fn hci_add_thing(
        &self, 
        reqeust: RequestCommon, 
        brand_id: String, 
        major_product_id: String, 
        minor_product_id: String,
        thing_mac: String, 
        thing_name: String, 
        thing_data: HashMap<String, String>
    ) -> NearResult<HciTaskId> {
        let hci_add_thing = Hci_add_thing {
            brand_id,
            major_product_id,
            minor_product_id,
            thing_name,
            thing: Some(Hci_thing {
                mac_address: thing_mac,
                data: thing_data,
                ..Default::default()
            }).into(),
            ..Default::default()
        };

        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_gateway::NEAR_THING_GATEWAY_ADD_THING_PUB.topic().clone(), 
            hci_add_thing
        )
        .await
    }

    pub async fn hci_crud_thing(
        &self, 
        reqeust: RequestCommon, 
        thing_id: String, 
        m: Hci_crud_m, 
        thing_data: HashMap<String, String>
    ) -> NearResult<HciTaskId> {
        let hci_op_thing = Hci_crud_thing {
            thing_id,
            method: m.into(),
            data: thing_data,
            ..Default::default()
        };

        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_gateway::NEAR_THING_GATEWAY_CRUD_THING_PUB.topic().clone(), 
            hci_op_thing
        )
        .await
    }

    pub async fn hci_ctrl_thing(
        &self, 
        reqeust: RequestCommon, 
        thing_id: String, 
        thing_data: HashMap<String, String>) -> NearResult<HciTaskId> {
        let hci_ctrl_thing = Hci_ctrl_thing {
            thing_id,
            data: thing_data,
            ..Default::default()
        };

        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_gateway::NEAR_THING_GATEWAY_CTRL_THING_PUB.topic().clone(), 
            hci_ctrl_thing
        )
        .await
    }

    pub async fn hci_get_task_result(
        &self, 
        reqeust: RequestCommon, 
        task_id: HciTaskId, 
        thing_ids: Option<Vec<String>>
    ) -> NearResult<Hci_thing_list> {

        self.request_and_wait(
            reqeust,
            topic_util::topics::hci_gateway::NEAR_THING_GATEWAY_SEARCH_RESULT_PUB.topic().clone(), 
            Hci_task_result {
            task_id,
            thing_ids: thing_ids.unwrap_or(vec![]),
            ..Default::default()
            }
        )
        .await
    }

}

impl ProcessTrait for CliStack {
    fn clone_as_process(&self) -> Box<dyn ProcessTrait> {
        Box::new(self.clone())
    }

    fn create_routine(&self, 
                      sender: &near_base::ObjectId, 
                      topic: &near_util::TopicRef) -> NearResult<Box<dyn near_transport::RoutineEventTrait>> {
        trace!("CliStack::create_routine, sender={sender}, topic={topic}");

        unimplemented!()
    }
}

impl ProcessEventTrait for CliStack {
    fn on_reinit(&self) {
        log::info!("FATAL!!!!!!!!!!!!: unimplemented CliStack::on_reinit");
    }
}

impl CliStack {
    async fn request_and_wait<REQ, RESP>(
        &self,
        request: RequestCommon,
        topic: Topic,
        req_raw_object: REQ, 
    ) -> NearResult<RESP>
    where REQ: Serialize + Deserialize + Send + Sync + std::default::Default + Clone + std::fmt::Display + 'static,
          RESP: Serialize + Deserialize + Send + Sync + std::default::Default + Clone + std::fmt::Display + 'static {
        clr_last_error();

        let req_raw_object = {
            protos::RawObjectHelper::encode_with_raw(req_raw_object)
                .map_err(| e | {
                    error!("failed build raw-object with err = {e}");
                    e
                })?
        };

        let (snd, rcv) = async_std::channel::bounded(1);
        let cb: Box<Cb<RESP>> = Cb::new(snd);

        self.stack()
            .post_message(
                RequestorMeta {
                    topic: Some(topic),
                    to: request.target,
                    ..Default::default()
                },
                req_raw_object, 
                Some(RoutineWrap::new(cb.clone()))
            )
            .await
            .map_err(| e | {
                error!("failed post-message with err = {e}");
                e
            })?;

        let _ = 
             rcv.recv()
                .await
                .map_err(| e | {
                    let error_string = format!("failed recv channel with err= {e}");
                    error!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_3RD, error_string)
                })?;

        match cb.take_result() {
            DataContent::Content(data) => {
                Ok(data)
            }
            DataContent::Error(err) => {
                error!("failed process with err: {err}");
                Err(err)
            }
        }
    }
}

