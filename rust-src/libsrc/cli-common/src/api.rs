
use std::path::PathBuf;

use log::error;
use near_base::{NearError, ErrorCode};
use protobuf::Message;

use crate::{HciOperator, desc};

#[derive(Default)]
pub struct ApiRequestCommon {
    pub target: Option<String>,
}

pub fn bm_desc_mnemonic(mnemonic: Option<String>) -> String {
    match desc::build_pk::set_mnemonic(mnemonic.as_ref().map(| v | v.as_str())) {
        Ok(v) => {
            v.unwrap_or_default()
        }
        Err(e) => {
            crate::stack::set_last_error(e);
            Default::default()
        }
    }
}

pub fn bm_desc_password(password: String) -> bool {
    desc::build_pk::set_password(password)
        .map_err(| e | {
            crate::stack::set_last_error(e);
        })
        .is_ok()
}

pub fn bm_desc_test_network() -> bool {
    desc::build_pk::set_test_network()
        .map_err(| e | {
            crate::stack::set_last_error(e);
        })
        .is_ok()
}

pub fn bm_desc_beta_network() -> bool {
    desc::build_pk::set_beta_network()
        .map_err(| e | {
            crate::stack::set_last_error(e);
        })
        .is_ok()
}

pub fn bm_desc_main_network() -> bool {
    desc::build_pk::set_main_network()
        .map_err(| e | {
            crate::stack::set_last_error(e);
        })
        .is_ok()
}

pub fn bm_desc_device_type() -> bool {
    false
}

pub fn bm_desc_people_type() -> bool {
    false
}

pub fn bm_desc_build(user_name: String, user_data: Vec<u8>, output_dir: String) -> Option<String> {
    desc::build(user_name, user_data, output_dir)
        .map_err(| e | {
            crate::stack::set_last_error(e);
        })
        .ok()
}

pub fn bm_search(bid: Option<String>) -> Vec<String> {
    match crate::search(bid) {
        Ok(data) => data,
        Err(e) => {
            crate::stack::set_last_error(e);
            vec![]
        }
    }
}

pub fn bm_init(near_home: Option<String>) -> bool {

    if let Err(e) = crate::init(None, near_home.map(| path | PathBuf::new().join(path) )) {
        crate::stack::set_last_error(e.clone());
        match e.errno() {
            ErrorCode::NEAR_ERROR_STARTUP => true,
            _ => false,
        }
    } else {
        true
    }
}

pub fn bm_open(people: String, core: String) -> bool {

    if let Err(e) = crate::open(people, core) {
        match e.errno() {
            ErrorCode::NEAR_ERROR_STARTUP => true,
            _ => false,
        }
    } else {
        true
    }
}

pub fn wait_online() -> bool {
    crate::wait_online()
}

pub fn get_last_error() -> String {
    crate::stack::get_last_error().to_string()
}

macro_rules! inner_bm_request {
    ($o: tt) => {
        match $o {
            Ok(v) => v.write_to_bytes()
            .map_err(| e | {
                let error_string = format!("failed encode to bytes with err = {e}");
                error!("{error_string}");
                NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string)
            }),
            Err(e) => Err(e),
        }
        .map_or_else(
        | e | {
            crate::stack::set_last_error(e);
            vec![]
        }, 
        | v | {
            v
        })
    };
}

pub fn bm_add_brand(reqeust: ApiRequestCommon, brand_name: String) -> Vec<u8> {
    inner_bm_request!({crate::add_brand(reqeust.into(), brand_name)})
}

pub fn bm_query_all_brand(reqeust: ApiRequestCommon, ) -> Vec<u8> {
    inner_bm_request!({crate::query_all_brand(reqeust.into(), )})
}

pub fn query_brand(reqeust: ApiRequestCommon, brand_id: String) -> Vec<u8> {
    inner_bm_request!({crate::query_brand(reqeust.into(), brand_id)})
}

pub fn remove_brand(reqeust: ApiRequestCommon, brand_id: String) -> bool {
    match crate::remove_brand(reqeust.into(), brand_id) {
        Ok(_) => true,
        Err(e) => {
            crate::stack::set_last_error(e);
            false
        }
    }
}

pub fn add_major_product(reqeust: ApiRequestCommon, product_name: String) -> Vec<u8> {
    inner_bm_request!({crate::add_major_product(reqeust.into(), product_name)})
}

pub fn add_minor_product(reqeust: ApiRequestCommon, major_product_id: String, product_name: String) -> Vec<u8> {
    inner_bm_request!({crate::add_minor_product(reqeust.into(), major_product_id, product_name)})
}

pub fn remove_major_product(reqeust: ApiRequestCommon, major_product_id: String) -> bool {
    match crate::remove_major_product(reqeust.into(), major_product_id) {
        Ok(_) => true,
        Err(e) => {
            crate::stack::set_last_error(e);
            false
        }
    }
}

pub fn remove_minor_product(reqeust: ApiRequestCommon, major_product_id: String, product_id: String) -> bool {
    match crate::remove_minor_product(reqeust.into(), major_product_id, product_id) {
        Ok(_) => true,
        Err(e) => {
            crate::stack::set_last_error(e);
            false
        }
    }
}

pub fn query_product(reqeust: ApiRequestCommon, product_id: String) -> Vec<u8> {
    inner_bm_request!({crate::query_product(reqeust.into(), product_id)})
}

pub fn query_all_product(reqeust: ApiRequestCommon, ) -> Vec<u8> {
    inner_bm_request!({crate::query_all_product(reqeust.into(), )})
}

pub fn update_thing(reqeust: ApiRequestCommon, thing_id: String, thing_name: String) -> Vec<u8> {
    inner_bm_request!({crate::update_thing(reqeust.into(), thing_id, thing_name)})
}

pub fn query_all_thing(reqeust: ApiRequestCommon, brand_id: Option<String>, product_id: Option<String>) -> Vec<u8> {
    inner_bm_request!({crate::query_all_thing(reqeust.into(), brand_id, product_id)})
}

// group
pub fn add_group(reqeust: ApiRequestCommon, schedule_name: String, schedule_img_index: Option<u32>) -> Vec<u8> {
    inner_bm_request!({crate::add_schedule(reqeust.into(), schedule_name, schedule_img_index, 0)})
}

pub fn add_maual_schedule(reqeust: ApiRequestCommon, schedule_name: String, schedule_img_index: Option<u32>) -> Vec<u8> {
    inner_bm_request!({crate::add_schedule(reqeust.into(), schedule_name, schedule_img_index, 1)})
}

pub fn add_timperiod_schedule(reqeust: ApiRequestCommon, schedule_name: String, schedule_img_index: Option<u32>) -> Vec<u8> {
    inner_bm_request!({crate::add_schedule(reqeust.into(), schedule_name, schedule_img_index, 2)})
}

pub fn update_schedule_property(reqeust: ApiRequestCommon, schedule_id: String, thing_relation: Vec<(String, Vec<(String, String)>)>) -> Vec<u8> {
    inner_bm_request!({crate::update_schedule_property(reqeust.into(), schedule_id, thing_relation)})
}

pub fn remove_schedule_property(reqeust: ApiRequestCommon, schedule_id: String, thing_relation: Vec<String>) -> Vec<u8> {
    inner_bm_request!({crate::remove_schedule_property(reqeust.into(), schedule_id, thing_relation)})
}

pub fn update_timeperiod_schedule_info(
    reqeust: ApiRequestCommon, 
    schedule_id: String,
    hour: u32, minute: u32,
    cycle_week_time: u32,
) -> Vec<u8> {
    inner_bm_request!({
        crate::update_timeperiod_schedule_info(
            reqeust.into(), 
            schedule_id, 
            hour, minute, 
            cycle_week_time
        )})
}

pub fn update_schedule_info(reqeust: ApiRequestCommon, schedule_id: String, schedule_name: String, schedule_img_index: Option<u32>) -> Vec<u8> {
    inner_bm_request!({
        crate::update_schedule_info(
            reqeust.into(), 
            schedule_id, 
            Some(schedule_name), 
            schedule_img_index, 
            None
        )})
}

pub fn enable_schedule(reqeust: ApiRequestCommon, schedule_id: String) -> Vec<u8> {
    inner_bm_request!({crate::update_schedule_info(reqeust.into(), schedule_id, None, None, Some(1))})
}

pub fn disable_schedule(reqeust: ApiRequestCommon, schedule_id: String) -> Vec<u8> {
    inner_bm_request!({crate::update_schedule_info(reqeust.into(), schedule_id, None, None, Some(2))})
}

pub fn remove_schedule(reqeust: ApiRequestCommon, schedule_id: String) -> bool {
    match crate::remove_schedule(reqeust.into(), schedule_id) {
        Ok(_) => true,
        Err(e) => {
            crate::stack::set_last_error(e);
            false
        }
    }
}

pub fn query_schedule(reqeust: ApiRequestCommon, schedule_id: String) -> Vec<u8> {
    inner_bm_request!({crate::query_schedule(reqeust.into(), schedule_id)})
}

pub fn query_all_simple_schedule(reqeust: ApiRequestCommon, ) -> Vec<u8> {
    inner_bm_request!({crate::query_all_simple_schedule(reqeust.into(), )})
}

pub fn execute_schedule(reqeust: ApiRequestCommon, schedule_id: String) -> bool {
    match crate::execute_schedule(reqeust.into(), schedule_id) {
        Ok(_) => true,
        Err(e) => {
            crate::stack::set_last_error(e);
            false
        }
    }
}

pub fn hci_search_thing(reqeust: ApiRequestCommon, brand_id: String) -> u32 {
    crate::hci_search_thing(reqeust.into(), brand_id)
}

pub fn hci_add_thing(
    reqeust: ApiRequestCommon, 
    brand_id: String, 
    major_product_id: String, 
    minor_product_id: String, 
    thing_mac: String, 
    thing_name: String, 
    thing_data: Vec<(String, String)>
) -> u32 {
    crate::hci_add_thing(
        reqeust.into(), 
        brand_id, 
        major_product_id, 
        minor_product_id, 
        thing_mac, 
        thing_name, 
        thing_data
    )
}

pub fn hci_remove_thing(reqeust: ApiRequestCommon, thing_id: String) -> u32 {
    crate::hci_crud_thing(reqeust.into(), thing_id, HciOperator::Remove, None)
}

pub fn hci_pair_thing(reqeust: ApiRequestCommon, thing_id: String, operator_data: Vec<(String, String)>) -> u32 {
    crate::hci_crud_thing(
        reqeust.into(), 
        thing_id, 
        HciOperator::Pair, 
        Some(operator_data)
    )
}

pub fn hci_removepair_thing(reqeust: ApiRequestCommon, thing_id: String, operator_data: Vec<(String, String)>) -> u32 {
    crate::hci_crud_thing(
        reqeust.into(), 
        thing_id, 
        HciOperator::RemovePair, 
        Some(operator_data)
    )
}

pub fn hci_query_thing(reqeust: ApiRequestCommon, thing_id: String) -> u32 {
    crate::hci_crud_thing(
        reqeust.into(), 
        thing_id, 
        HciOperator::Query, 
        None
    )
}

pub fn hci_ctrl_thing(reqeust: ApiRequestCommon, thing_id: String, operator_data: Vec<(String, String)>) -> u32 {
    crate::hci_ctrl_thing(
        reqeust.into(), 
        thing_id, 
        operator_data
    )
}

pub fn hci_get_task_result(reqeust: ApiRequestCommon, task_id: u32, thing_ids: Vec<String>) -> Vec<u8> {
    inner_bm_request!({
        crate::hci_get_task_result(
            reqeust.into(), 
            task_id, 
            if thing_ids.len() == 0 { None } else { Some(thing_ids) }
        )})
}

mod test{

    #[test]
    fn test_init() {
        if crate::api::bm_init(Some("x:\\abc".to_owned())) == true {
            return;
        } else {
            let err = crate::api::get_last_error();
            println!("{err}");
        }
    }
}
