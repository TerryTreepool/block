pub mod api;
#[cfg(any(target_os = "android", target_os = "ios"))]
pub mod bridge_generated;
pub mod cb;
pub mod desc;
pub mod stack;
pub mod test_brand;

use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use log::{debug, error, trace};
use near_util::{DESC_SUFFIX_NAME, KEY_SUFFIX_NAME};
use std::path::PathBuf;

use near_base::{
    people::PeopleObject, DeviceObject, ErrorCode, FileDecoder, NearError, NearResult,
};
use near_base::{FileEncoder, ObjectId, PrivateKey};
use near_core::path_utils::alter_near_path;
use near_core::{alter_root_path, get_temp_path, LogLevel};
use near_core::{get_data_path, get_log_path, panic::PanicBuilder, LoggerBuilder};
use once_cell::sync::OnceCell;
use protos::hci::brand::*;
use protos::hci::hci_thing::{hci_crud_thing::Hci_crud_m, *};
use protos::hci::product::*;
use protos::hci::schedule::*;
use protos::hci::thing::*;
use stack::CliStackBuild;
use topic_util::types::hci_types::HciTaskId;

use crate::api::ApiRequestCommon;
use crate::stack::CliStack;

pub struct CliCommonConfig {
    pub(crate) timeout: Duration,
}

#[derive(Default)]
pub struct RequestCommon {
    target: Option<ObjectId>,
}

impl From<ApiRequestCommon> for RequestCommon {
    fn from(value: ApiRequestCommon) -> Self {

        Self {
            target: {
                match value.target {
                    Some(target) => {
                        if let Ok(target) = ObjectId::from_str(&target) {
                            Some(target)
                        } else {
                            None
                        }
                    }
                    None => None,
                }
            },
        }
    }
}

impl std::fmt::Display for RequestCommon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[target: {{{:?}}}]", self.target.as_ref())
    }
}

impl std::default::Default for CliCommonConfig {
    fn default() -> Self {
        Self {
            timeout: {
                #[cfg(debug_assertions)]
                {
                    Duration::from_secs(1000)
                }

                #[cfg(not(debug_assertions))]
                {
                    Duration::from_secs(10)
                }
            },
        }
    }
}

impl CliCommonConfig {
    pub(crate) fn get_instance() -> &'static CliCommonConfig {
        static INSTANCE: OnceCell<CliCommonConfig> = OnceCell::new();

        INSTANCE.get_or_init(|| CliCommonConfig::default())
    }
}

pub fn get_last_error() -> String {
    let (_, error_string) = stack::get_last_error().split();

    error_string.unwrap_or(String::default())
}

pub fn search(id: Option<String>) -> NearResult<Vec<String>> {
    let id = match id.as_ref() {
        Some(id) => Some(ObjectId::from_str(id)?),
        None => None,
    };

    let r = async_std::task::block_on(async move {
        let probe = discovery_util::c::probe::Probe::new(None);

        probe.run().await?;

        probe.wait().await
    })?;

    let mut dataes = vec![];
    r.desc_list
        .iter()
        .filter(|desc| {
            if let Some(id) = id.as_ref() {
                desc.object_id() == id
            } else {
                true
            }
        })
        .for_each(|desc| {
            let path = get_temp_path().join(
                PathBuf::new()
                    .with_file_name(desc.object_id().to_string())
                    .with_extension(DESC_SUFFIX_NAME),
            );
            if let Ok(_) = desc.encode_to_file(path.as_path(), false) {
                dataes.push(path.display().to_string());
            }
        });

    Ok(dataes)
}

pub fn init(log_level: Option<LogLevel>, near_home: Option<PathBuf>) -> NearResult<()> {
    let near_path = if let Some(path) = near_home {
        if !path.exists() {
            Err(NearError::new(
                ErrorCode::NEAR_ERROR_DONOT_EXIST,
                format!("{} not exist.", path.display()),
            ))
        } else {
            Ok(Some(path))
        }
    } else {
        Ok(None)
    }?;

    if let Some(near_path) = near_path {
        alter_root_path(near_path)
    } else {
        alter_near_path()
    }?;

    let log_level = log_level.unwrap_or(near_core::LogLevel::Trace);

    // build log
    let _ = 
        LoggerBuilder::new("cli-common", get_log_path().join("cli-common"))
            .set_level(log_level)
            .set_console(log_level)
            .build()
            .map_err(|e| {
                error!("failed build logger egine with e={e}");
                e
            })?;

    let _ = 
        PanicBuilder::new("cli-common")
            .exit_on_panic(true)
            .log_to_file(true)
            .build()
            .start();

    Ok(())
}

pub fn open(people: String, core: String) -> NearResult<()> {
    if let Some(_) = CliStack::get_instance() {
        Err(NearError::new(ErrorCode::NEAR_ERROR_STARTUP, "startup"))
    } else {
        Ok(())
    }?;

    debug!("people: {people}");
    debug!("core: {core}");

    let people_desc =
        PeopleObject::decode_from_file(get_data_path().join(format!("{people}.{DESC_SUFFIX_NAME}")).as_path())
            .map_err(|e| {
                error!("failed decode people object with err={e}");
                e
            })?;

    let people_key =
        PrivateKey::decode_from_file(get_data_path().join(format!("{people}.{KEY_SUFFIX_NAME}")).as_path())
            .map_err(|e| {
                error!("failed decode people key with err={e}");
                e
            })?;

    let core =
        DeviceObject::decode_from_file(get_data_path().join(format!("{core}.{DESC_SUFFIX_NAME}")).as_path())
            .map_err(|e| {
                error!("failed decode core-service object with err={e}");
                e
            })?;

    async_std::task::block_on(async move {
        let init_f = async move {
            let r = CliStackBuild {
                people: people_desc,
                people_private_key: people_key,
                core,
            }
            .build()
            .await
            .map_err(|e| {
                error!("failed build cli-stack with err = {e}");
                e
            });

            r
        };

        match async_std::future::timeout(CliCommonConfig::get_instance().timeout, init_f).await {
            Ok(r) => r,
            Err(e) => {
                let error_string = format!("init cli-stack is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

pub fn wait_online() -> bool {
    trace!("wait onine");

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance().unwrap().wait_online(),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                let error_string = format!("wait online is timeout, err = {e}");
                error!("{error_string}");
                return false;
            }
        }
    })
}

pub fn add_brand(reqeust: RequestCommon, brand_name: String) -> NearResult<Brand_info> {
    trace!("reqeust: {reqeust}, brand_name: {brand_name}");

    let new_brand = Brand_info {
        brand_name,
        ..Default::default()
    };

    let r = async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance().unwrap().add_brand(reqeust, new_brand),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                let error_string = format!("init add_brand is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })?;

    Ok(r)
}

pub fn query_all_brand(reqeust: RequestCommon, ) -> NearResult<Brand_info_list> {
    let r = async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance().unwrap().query_all_brand(reqeust),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("query_all_brand is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })?;

    Ok(r)
}

pub fn query_brand(reqeust: RequestCommon, brand_id: String) -> NearResult<Brand_info> {
    trace!("brand_id: {brand_id}");

    let r = async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance().unwrap().query_brand(reqeust, brand_id),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("query_brand is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })?;

    Ok(r)
}

pub fn remove_brand(reqeust: RequestCommon, brand_id: String) -> NearResult<()> {
    trace!("brand_id: {brand_id}");

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance().unwrap().remove_brand(reqeust, brand_id),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("query_brand is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })?;

    Ok(())
}

pub fn add_major_product(reqeust: RequestCommon, product_name: String) -> NearResult<Product_info> {
    trace!("add_product: product_name: {product_name}");

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .add_product(reqeust, None, product_name),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("add product is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

pub fn add_minor_product(
    reqeust: RequestCommon, 
    major_product_id: String,
    product_name: String,
) -> NearResult<Product_info> {
    trace!("add_product: major_product_id: {major_product_id}, product_name: {product_name}");

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .add_product(reqeust, Some(major_product_id), product_name),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("add product is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

pub fn remove_major_product(reqeust: RequestCommon, major_product_id: String) -> NearResult<()> {
    trace!("remove_major_product: product_id: {major_product_id}");

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .remove_product(reqeust, None, major_product_id),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("update_product is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })?;

    Ok(())
}

pub fn remove_minor_product(reqeust: RequestCommon, major_product_id: String, product_id: String) -> NearResult<()> {
    trace!("remove_minor_product: product_id: {product_id}");

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .remove_product(reqeust, Some(major_product_id), product_id),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("update_product is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })?;

    Ok(())
}

pub fn query_product(reqeust: RequestCommon, product_id: String) -> NearResult<Product_info> {
    trace!("query_product: product_id: {product_id}");

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance().unwrap().query_product(reqeust, product_id),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("query_product is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

pub fn query_all_product(reqeust: RequestCommon, ) -> NearResult<Product_info_list> {
    trace!("query_all_product");

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance().unwrap().query_all_product(reqeust, ),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("query_all_product is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

// pub fn add_device(product_id: String, device_mac_address: String, device_name: String) -> NearResult<Device_info> {
//     trace!("add_device: product_id: {product_id}, device_mac_address:{device_mac_address}, device_name: {device_name}");

//     async_std::task::block_on(async move {
//         match async_std::future::timeout(CliCommonConfig::get_instance().timeout,
//                                          CliStack::get_instance().unwrap().add_device(product_id, device_mac_address, device_name))
//                 .await {
//             Ok(data) => data,
//             Err(e) => {
//                 let error_string = format!("add_device is timeout, err = {e}");
//                 error!("{error_string}");
//                 Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
//             }
//         }
//     })
// }

pub fn update_thing(
    reqeust: RequestCommon, 
    thing_id: String, 
    thing_name: String
) -> NearResult<Thing_info> {
    trace!(
        "update_thing: thing_id: {thing_id}, thing_name:{:?}",
        thing_name
    );

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .update_thing(reqeust, thing_id, thing_name),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("update_thing is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

// pub fn query_device(device_id: String) -> NearResult<Device_info> {
//     trace!("query_device: device_id: {device_id}");

//     async_std::task::block_on(async move {
//         match async_std::future::timeout(CliCommonConfig::get_instance().timeout,
//                                          CliStack::get_instance().unwrap().query_device(device_id))
//                 .await {
//             Ok(data) => data,
//             Err(e) => {
//                 let error_string = format!("query_device is timeout, err = {e}");
//                 error!("{error_string}");
//                 Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
//             }
//         }
//     })
// }

pub fn query_all_thing(
    reqeust: RequestCommon, 
    brand_id: Option<String>,
    product_id: Option<String>,
) -> NearResult<Thing_info_list> {
    trace!(
        "query_all_thing: brand_id: {:?}, product_id: {:?}",
        brand_id,
        product_id
    );

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .query_all_thing(reqeust, brand_id, product_id),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("query_all_thing is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

// schedule
pub fn add_schedule(
    reqeust: RequestCommon, 
    schedule_name: String,
    schedule_img_index: Option<u32>,
    schedule_mode: i32,
) -> NearResult<Schedule_info> {
    trace!("add_schedule: schedule_name: {schedule_name}");

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance().unwrap().add_schedule(
                reqeust, 
                schedule_name,
                schedule_img_index,
                schedule_mode,
                vec![],
            ),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("add_schedule is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

pub fn update_schedule_property(
    reqeust: RequestCommon, 
    schedule_id: String,
    thing_relation: Vec<(String, Vec<(String, String)>)>,
) -> NearResult<Schedule_info> {
    trace!(
        "update_schedule_property: schedule_id: {schedule_id}, thing_relation: {:?}",
        thing_relation
    );

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .update_schedule_property(reqeust, schedule_id, thing_relation),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("update_schedule_property is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

pub fn remove_schedule_property(
    reqeust: RequestCommon, 
    schedule_id: String,
    thing_relation: Vec<String>,
) -> NearResult<Schedule_info> {
    trace!(
        "remove_schedule_property: schedule_id: {schedule_id}, thing_relation: {:?}",
        thing_relation
    );

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .remove_schedule_property(reqeust, schedule_id, thing_relation),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("remove_schedule_property is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

pub fn update_timeperiod_schedule_info(
    reqeust: RequestCommon, 
    schedule_id: String,
    hour: u32,
    minute: u32,
    cycle_week: u32,
) -> NearResult<Schedule_info> {
    trace!("update_timeperiod_schedule_info: schedule_id: {schedule_id}, time: {{{hour}:{minute}}}, cycle_week: {cycle_week}");

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .update_timeperiod_schedule_info(reqeust, schedule_id, hour, minute, cycle_week),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("update_timeperiod_schedule_info is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

pub fn update_schedule_info(
    reqeust: RequestCommon, 
    schedule_id: String,
    schedule_name: Option<String>,
    schedule_img_index: Option<u32>,
    schedule_status: Option<u32>,
) -> NearResult<Schedule_info> {
    trace!(
        "update_schedule: schedule_id: {schedule_id}, schedule_status: {:?}, schedule_name: {:?}",
        schedule_status,
        schedule_name
    );

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance().unwrap().update_schedule_info(
                reqeust, 
                schedule_id,
                schedule_name,
                schedule_img_index,
                schedule_status,
            ),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("update_schedule is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

pub fn remove_schedule(
    reqeust: RequestCommon, 
    schedule_id: String
) -> NearResult<()> {
    trace!("remove_schedule: schedule_id: {schedule_id}");

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .remove_schedule(reqeust, schedule_id),
        )
        .await
        {
            Ok(data) => data.map(|_| ()),
            Err(e) => {
                let error_string = format!("remove_schedule is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

pub fn query_schedule(
    reqeust: RequestCommon, 
    schedule_id: String
) -> NearResult<Schedule_info> {
    trace!("query_schedule: schedule_id: {schedule_id}");

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .query_schedule(reqeust, schedule_id),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("query_schedule is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

pub fn query_all_simple_schedule(reqeust: RequestCommon, ) -> NearResult<Schedule_list> {
    trace!("query_all_simple_schedule");

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .query_all_simple_schedule(reqeust, ),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("query_all_simple_schedule is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

pub fn execute_schedule(reqeust: RequestCommon, schedule_id: String) -> NearResult<()> {
    trace!("execute_schedule: schedule_id: {schedule_id}");

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .execute_schedule(reqeust, schedule_id),
        )
        .await
        {
            Ok(data) => data.map(|_| ()),
            Err(e) => {
                let error_string = format!("execute_schedule is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

pub enum HciOperator {
    Remove = 0,
    Pair = 1,
    RemovePair = 2,
    Query = 3,
}

impl std::fmt::Display for HciOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            Self::Remove => "remove",
            Self::Pair => "pair",
            Self::RemovePair => "remove-pair",
            Self::Query => "query",
        };

        write!(f, "{v}")
    }
}

impl From<Hci_crud_m> for HciOperator {
    fn from(value: Hci_crud_m) -> Self {
        match value {
            Hci_crud_m::remove => Self::Remove,
            Hci_crud_m::pair => Self::Pair,
            Hci_crud_m::query => Self::Query,
            Hci_crud_m::remove_pair => Self::RemovePair,
        }
    }
}

impl From<HciOperator> for Hci_crud_m {
    fn from(value: HciOperator) -> Self {
        match value {
            HciOperator::Remove => Self::remove,
            HciOperator::Pair => Self::pair,
            HciOperator::Query => Self::query,
            HciOperator::RemovePair => Self::remove_pair,
        }
    }
}

pub fn hci_search_thing(reqeust: RequestCommon, brand_id: String) -> HciTaskId {
    trace!("start_search_device: brand_id: {}", brand_id);

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance().unwrap().hci_search_thing(reqeust, brand_id),
        )
        .await
        {
            Ok(data) => data.ok().unwrap_or(0),
            Err(e) => {
                let error_string = format!("hci_search_thing is timeout, err = {e}");
                error!("{error_string}");
                crate::stack::set_last_error(NearError::new(
                    ErrorCode::NEAR_ERROR_TIMEOUT,
                    error_string,
                ));
                0
            }
        }
    })
}

pub fn hci_add_thing(
    reqeust: RequestCommon, 
    brand_id: String,
    major_product_id: String,
    minor_product_id: String,
    thing_mac: String,
    thing_name: String,
    thing_data: Vec<(String, String)>,
) -> HciTaskId {
    trace!("hci_add_thing: brand_id: {brand_id}, product_id: {major_product_id}-{minor_product_id}, thing_mac: {thing_mac}, thing_name: {thing_name}, thing_data count: {}", thing_data.len());

    let mut map = HashMap::new();
    for (k, v) in thing_data {
        let _ = map.insert(k, v);
    }

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance().unwrap().hci_add_thing(
                reqeust, 
                brand_id,
                major_product_id,
                minor_product_id,
                thing_mac,
                thing_name,
                map,
            ),
        )
        .await
        {
            Ok(data) => data.ok().unwrap_or(0),
            Err(e) => {
                let error_string = format!("hci_add_thing is timeout, err = {e}");
                error!("{error_string}");
                crate::stack::set_last_error(NearError::new(
                    ErrorCode::NEAR_ERROR_TIMEOUT,
                    error_string,
                ));
                0
            }
        }
    })
}

pub fn hci_crud_thing(
    reqeust: RequestCommon, 
    thing_id: String,
    m: HciOperator,
    thing_data: Option<Vec<(String, String)>>,
) -> HciTaskId {
    trace!("hci_crud_thing: thing_id: {thing_id}, operator: {m}",);

    let mut data = HashMap::new();
    for (k, v) in thing_data.unwrap_or_default() {
        let _ = data.insert(k, v);
    }

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .hci_crud_thing(reqeust, thing_id, m.into(), data),
        )
        .await
        {
            Ok(data) => data.ok().unwrap_or(0),
            Err(e) => {
                let error_string = format!("hci_crud_thing is timeout, err = {e}");
                error!("{error_string}");
                crate::stack::set_last_error(NearError::new(
                    ErrorCode::NEAR_ERROR_TIMEOUT,
                    error_string,
                ));
                0
            }
        }
    })
}

pub fn hci_ctrl_thing(
    reqeust: RequestCommon, 
    thing_id: String, 
    thing_data: Vec<(String, String)>
) -> HciTaskId {
    trace!(
        "hci_ctrl_thing: thing_id: {thing_id}, thing_data count: {}",
        thing_data.len()
    );

    let mut map = HashMap::new();
    for (k, v) in thing_data {
        let _ = map.insert(k, v);
    }

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .hci_ctrl_thing(reqeust, thing_id, map),
        )
        .await
        {
            Ok(data) => data.ok().unwrap_or(0),
            Err(e) => {
                let error_string = format!("hci_ctrl_thing is timeout, err = {e}");
                error!("{error_string}");
                crate::stack::set_last_error(NearError::new(
                    ErrorCode::NEAR_ERROR_TIMEOUT,
                    error_string,
                ));
                0
            }
        }
    })
}

pub fn hci_get_task_result(
    reqeust: RequestCommon, 
    task_id: HciTaskId,
    thing_ids: Option<Vec<String>>,
) -> NearResult<Hci_thing_list> {
    trace!("get_search_result: task_id: {}", task_id);

    async_std::task::block_on(async move {
        match async_std::future::timeout(
            CliCommonConfig::get_instance().timeout,
            CliStack::get_instance()
                .unwrap()
                .hci_get_task_result(reqeust, task_id, thing_ids),
        )
        .await
        {
            Ok(data) => data,
            Err(e) => {
                let error_string = format!("hci_get_task_result is timeout, err = {e}");
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, error_string))
            }
        }
    })
}

#[cfg(test)]
#[allow(unused)]
mod test {
    use async_std::fs::create_dir_all;
    use near_base::{ErrorCode, NearError, NearResult};
    use near_core::LogLevel;

    use crate::{api::ApiRequestCommon, CliCommonConfig, RequestCommon};

    // static CS: &'static str = "core-service_116.205.227.113";
    // static CS: &'static str = "core-service";
    static CS: &'static str = "core-service";

    pub fn global_print<T: std::fmt::Display>(r: NearResult<T>) {
        println!("++++++++++++++++++++++++++++++++++++++++++++++");
        match r {
            Ok(r) => println!("value={r}"),
            Err(e) => println!("e={e}"),
        }
        println!("++++++++++++++++++++++++++++++++++++++++++++++");
    }

    pub fn global_print_error(r: NearError) {
        println!("**********************************************");
        println!("e={r}");
        println!("**********************************************");
    }

    #[test]
    pub fn test_add_brand() -> NearResult<()> {
        let r = async_std::task::block_on(async move {
            crate::init(Some(LogLevel::Info), None).unwrap();
            crate::open("BM".to_owned(), CS.to_owned()).unwrap();
            // crate::open("7zz4SFi7McFc5HWPuFNY2MYp8KdiT7eHWdcjBRqLyRNp".to_owned(), "3hAXN4eThpDfLuL9m5Uty5MVibbx3pcyY2cQpfBHEgK1".to_owned()).unwrap();

            println!("begin");
            let now = near_base::now();
            // global_print(crate::add_brand("test1".to_owned()));
            // global_print(crate::add_brand("test2".to_owned()));
            // global_print(crate::add_brand("vanhai".to_owned()));
            let mut futs = vec![];
            for _ in 0..10 {
                futs.push(async_std::task::spawn(async move {
                    crate::query_all_brand(ApiRequestCommon::default().into());
                }));
            }

            let _ = futures::future::join_all(futs).await;

            let end = near_base::now();
            println!("+_+_+_+_+_+_+_+_+_+_second: {}", std::time::Duration::from_micros(end-now).as_secs());
            // global_print(crate::query_brand("5EEB593E0968AF0FC01260808EC79ABC".to_owned()));
            // if let Err(e) = crate::remove_brand("5EEB593E0968AF0FC01260808EC79ABC".to_owned()) {
            //     global_print_error(e)
            // }
            println!("{}", crate::wait_online());
        });

        Ok(())
    }

    #[test]
    pub fn test_add_product() -> NearResult<()> {
        let r = async_std::task::block_on(async move {
            // crate::init(None, None).unwrap();
            // // crate::init("BM".to_owned(), CS.to_owned()).unwrap();

            // global_print(crate::add_major_product("test1".to_owned()));
            // global_print(crate::add_major_product("test2".to_owned()));
            // global_print(crate::add_major_product("test3".to_owned()));
            // global_print(crate::add_minor_product("16D69E393F9DFA12CE56B2B81BA20BED".to_owned(), "cw1".to_owned()));
            // global_print(crate::add_minor_product("16D69E393F9DFA12CE56B2B81BA20BED".to_owned(), "cw2".to_owned()));
            // global_print(crate::add_minor_product("16D69E393F9DFA12CE56B2B81BA20BED".to_owned(), "cw3".to_owned()));
            // global_print(crate::query_product("16D69E393F9DFA12CE56B2B81BA20BED".to_owned()));

            // global_print(crate::query_all_product());

            // if let Err(e) = crate::remove_minor_product("16D69E393F9DFA12CE56B2B81BA20BED".to_owned(), "74B47AA542A07A8C3BFEEB029898D986".to_owned()) {
            //     global_print_error(e);
            // }
            // global_print(crate::query_product("16D69E393F9DFA12CE56B2B81BA20BED".to_owned()));
            // global_print(crate::query_all_product());

            // if let Err(e) = crate::remove_major_product("16D69E393F9DFA12CE56B2B81BA20BED".to_owned()) {
            //     global_print_error(e);
            // }
            // global_print(crate::query_product("16D69E393F9DFA12CE56B2B81BA20BED".to_owned()));
            // global_print(crate::query_all_product());

            // // global_print(crate::remove_minor_product("16D69E393F9DFA12CE56B2B81BA20BED".to_owned(), "cw3".to_owned()));

            // // global_print(crate::add_brand("test2".to_owned()));
            // // global_print(crate::add_brand("vanhai".to_owned()));
            // // global_print(crate::query_all_brand());
            // // global_print(crate::query_brand("5EEB593E0968AF0FC01260808EC79ABC".to_owned()));
            // // if let Err(e) = crate::remove_brand("5EEB593E0968AF0FC01260808EC79ABC".to_owned()) {
            // //     global_print_error(e)
            // // }
        });

        Ok(())
    }

    #[test]
    pub fn test_group() -> NearResult<()> {
        // add group
        {
            async_std::task::block_on(async move {
                // crate::init("BM".to_owned(), CS.to_owned()).unwrap();

                // // global_print(crate::add_schedule("group-test".to_owned(), Some(1), 1));
                // // global_print(crate::query_all_simple_schedule());
                // {
                //     // let data = vec![("1".to_owned(), "a".to_owned()), ("2".to_owned(), "b".to_owned())];
                //     // global_print(crate::update_schedule("0CBC1B65E5D24BA3D6231B18380A881F".to_owned(), None, None, None, Some(vec![("1234".to_owned(), data)])));
                //     global_print(crate::update_schedule_info(
                //         "5FAC088815756AE15DADD1641A44A6F8".to_owned(),
                //         None,
                //         Some(22),
                //         None)
                //     )
                // }
                // {
                //     global_print(crate::add_schedule(
                //         "abc".to_owned(),
                //         Some(10),
                //         2));
                // }
                // {
                //     global_print(crate::update_timeperiod_schedule_info(
                //         "69DA530C1BF1BE3C6C006D0FEF2AD9EB".to_owned(),
                //         23, 32,
                //         0));
                // }

                // {
                //     let relation = vec!["5Se27iyThejRTmrvRha4cb3JTBbubwYMv9ejMs1paUhh".to_owned()];
                //     global_print(crate::remove_schedule_property(
                //         "5FAC088815756AE15DADD1641A44A6F8".to_owned(), relation))
                // }
                // // global_print(crate::query_schedule("0CBC1B65E5D24BA3D6231B18380A881F".to_owned()));
            });
        }

        Ok(())
    }

    #[test]
    pub fn test_query_all_brand() -> NearResult<()> {
        let r = {
            // crate::init("BM".to_owned(), CS.to_owned())?;

            // for brand in crate::query_all_brand()?.brands() {
            //     println!("+++++++++++++++++++++++++++++++++++++++");
            //     println!("{}", brand);
            // }

            Ok(())
        };

        r
    }

    // #[test]
    // pub fn test_query_all_thing() -> NearResult<()> {
    //     let r = {
    //         crate::init("BM".to_owned(), CS.to_owned(), None)?;

    //         for brand in crate::query_all_device(None)?.devices {
    //             println!("+++++++++++++++++++++++++++++++++++++++");
    //             println!("{}", brand);
    //         }

    //         Ok(())
    //     };

    //     r
    // }

    #[test]
    pub fn test_search_thing() -> NearResult<()> {
        let r = {
                crate::init(Some(LogLevel::Info), None).unwrap();
                crate::open("BM".to_owned(), CS.to_owned()).unwrap();
                // crate::init("BM".to_owned(), CS.to_owned(),)?;

                if !crate::wait_online() {
                    panic!("failed online");
                }

                let task_id = crate::hci_search_thing(RequestCommon::default(), "5EEB593E0968AF0FC01260808EC79ABCdd".to_owned());

                if task_id == 0 {
                    panic!("failed search");
                } else {
                    println!("*************************************{task_id}");
                }

            for _ in 0..2 {
                    println!("+++++++++++++++++++++++++++++++++++++++");
                    std::thread::sleep(std::time::Duration::from_secs(10));

                    match crate::hci_get_task_result(RequestCommon::default(), task_id, None) {
                        Ok(v) => {
                            for i in v.list() {
                                println!(r#"
                                ---------------------------------------
                                {},
                                dataes:{:?},
                                ---------------------------------------"#,
                                i.mac_address,
                                i.data);
                            }
                        }
                        Err(e) => {
                            println!("{e}");
                        }
                    }
                }

            Ok(())
        };

        r
    }
    // #[test]
    // pub fn test_query_brand() -> NearResult<()> {
    //     let r = {
    //         crate::init("BM".to_owned(), CS.to_owned())?;

    //         let brand = crate::query_brand("BB4161CE37466CEBB4C055C7664C071F".to_owned())?;

    //         println!("{brand}");

    //         Ok(())
    //     };

    //     r
    // }

    // #[test]
    // pub fn test_update_brand() -> NearResult<()> {
    //     let r = {
    //         crate::init("BM".to_owned(), CS.to_owned())?;

    //         let brand = crate::update_brand_status("5641C3F6E0E4FB25A04B746DBB9670E2".to_owned(), 10)?;

    //         println!("{brand}");

    //         Ok(())
    //     };

    //     r
    // }

    // #[test]
    // fn test_product() {

    //     async_std::task::block_on(async move {

    //         crate::init("BM".to_owned(), CS.to_owned()).unwrap();

    //             // add
    //         let r = crate::add_product("5641C3F6E0E4FB25A04B746DBB9670E2".to_owned(),
    //                            "test_product_a".to_owned(),
    //                            1);

    //         println!("++++++++++++++++++++++++++++++++++++++++++++++");
    //         println!("{:?}", r);

    //         std::thread::sleep(std::time::Duration::from_secs(1));
    //         // query
    //         let r = crate::query_all_product(None);
    //         println!("++++++++++++++++++++++++++++++++++++++++++++++");
    //         println!("{:?}", r);

    //         std::thread::sleep(std::time::Duration::from_secs(1));

    //         if let Some(a) = r.unwrap().pop() {
    //             let r = crate::query_product(a.product_id);

    //             println!("++++++++++++++++++++++++++++++++++++++++++++++");
    //             println!("{:?}", r);
    //             std::thread::sleep(std::time::Duration::from_secs(1));
    //         }

    //     })
    // }

    // #[test]
    // fn test_device() {

    //     async_std::task::block_on(async move {

    //         crate::init("BM".to_owned(), CS.to_owned()).unwrap();

    //             // add
    //         let r = crate::add_device("4D9E67D78509139B1C7329E5F53587C9".to_owned(),
    //                             "00-50-56-C0-00-08".to_owned(),
    //                            "test_device_a".to_owned(),
    //                            1);

    //         println!("++++++++++++++++++++++++++++++++++++++++++++++");
    //         println!("{:?}", r);

    //         std::thread::sleep(std::time::Duration::from_secs(1));
    //         // query
    //         let r = crate::query_all_device(None);
    //         println!("++++++++++++++++++++++++++++++++++++++++++++++");
    //         println!("{:?}", r);

    //         std::thread::sleep(std::time::Duration::from_secs(1));

    //         if let Some(a) = r.unwrap().pop() {
    //             let r = crate::query_device(a.product_id);

    //             println!("++++++++++++++++++++++++++++++++++++++++++++++");
    //             println!("{:?}", r);
    //             std::thread::sleep(std::time::Duration::from_secs(1));
    //         }

    //     })
    // }

    #[test]
    fn test_callpeer() {

        async_std::task::block_on(async move {

            // crate::init("BM".to_owned(), CS.to_owned()).unwrap();
            crate::init(None, None).unwrap();
            crate::open("BM".to_owned(), CS.to_owned()).unwrap();

            let r = 
                crate::query_all_product(
                    ApiRequestCommon {
                        target: Some("3hAyrhz5WGuAuov6kRCPFyxC876yin88Ybd72uaiti3n".to_owned()),
                    }.into(), 
                )
                .unwrap();
            println!("{:?}", r);
            // // add
            // let r = crate::add_device("4D9E67D78509139B1C7329E5F53587C9".to_owned(),
            //                     "00-50-56-C0-00-08".to_owned(),
            //                    "test_device_a".to_owned(),
            //                    1);

            println!("++++++++++++++++++++++++++++++++++++++++++++++");
            // let r = crate::query_brand(brand_id)

            // std::thread::sleep(std::time::Duration::from_secs(1));
            // // query
            // let r = crate::query_all_device(None);
            // println!("++++++++++++++++++++++++++++++++++++++++++++++");
            // println!("{:?}", r);

            // std::thread::sleep(std::time::Duration::from_secs(1));

            // if let Some(a) = r.unwrap().pop() {
            //     let r = crate::query_device(a.product_id);

            //     println!("++++++++++++++++++++++++++++++++++++++++++++++");
            //     println!("{:?}", r);
            //     std::thread::sleep(std::time::Duration::from_secs(1));
            // }

        })
    }
}
