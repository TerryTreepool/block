use super::*;
// Section: wire functions

#[no_mangle]
pub extern "C" fn wire_bm_desc_mnemonic(port_: i64, mnemonic: *mut wire_uint_8_list) {
    wire_bm_desc_mnemonic_impl(port_, mnemonic)
}

#[no_mangle]
pub extern "C" fn wire_bm_desc_password(port_: i64, password: *mut wire_uint_8_list) {
    wire_bm_desc_password_impl(port_, password)
}

#[no_mangle]
pub extern "C" fn wire_bm_desc_test_network(port_: i64) {
    wire_bm_desc_test_network_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_bm_desc_beta_network(port_: i64) {
    wire_bm_desc_beta_network_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_bm_desc_main_network(port_: i64) {
    wire_bm_desc_main_network_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_bm_desc_device_type(port_: i64) {
    wire_bm_desc_device_type_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_bm_desc_people_type(port_: i64) {
    wire_bm_desc_people_type_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_bm_desc_build(
    port_: i64,
    user_name: *mut wire_uint_8_list,
    user_data: *mut wire_uint_8_list,
    output_dir: *mut wire_uint_8_list,
) {
    wire_bm_desc_build_impl(port_, user_name, user_data, output_dir)
}

#[no_mangle]
pub extern "C" fn wire_bm_search(port_: i64, bid: *mut wire_uint_8_list) {
    wire_bm_search_impl(port_, bid)
}

#[no_mangle]
pub extern "C" fn wire_bm_init(port_: i64, near_home: *mut wire_uint_8_list) {
    wire_bm_init_impl(port_, near_home)
}

#[no_mangle]
pub extern "C" fn wire_bm_open(
    port_: i64,
    people: *mut wire_uint_8_list,
    core: *mut wire_uint_8_list,
) {
    wire_bm_open_impl(port_, people, core)
}

#[no_mangle]
pub extern "C" fn wire_wait_online(port_: i64) {
    wire_wait_online_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_get_last_error(port_: i64) {
    wire_get_last_error_impl(port_)
}

#[no_mangle]
pub extern "C" fn wire_bm_add_brand(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    brand_name: *mut wire_uint_8_list,
) {
    wire_bm_add_brand_impl(port_, reqeust, brand_name)
}

#[no_mangle]
pub extern "C" fn wire_bm_query_all_brand(port_: i64, reqeust: *mut wire_ApiRequestCommon) {
    wire_bm_query_all_brand_impl(port_, reqeust)
}

#[no_mangle]
pub extern "C" fn wire_query_brand(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    brand_id: *mut wire_uint_8_list,
) {
    wire_query_brand_impl(port_, reqeust, brand_id)
}

#[no_mangle]
pub extern "C" fn wire_remove_brand(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    brand_id: *mut wire_uint_8_list,
) {
    wire_remove_brand_impl(port_, reqeust, brand_id)
}

#[no_mangle]
pub extern "C" fn wire_add_major_product(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    product_name: *mut wire_uint_8_list,
) {
    wire_add_major_product_impl(port_, reqeust, product_name)
}

#[no_mangle]
pub extern "C" fn wire_add_minor_product(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    major_product_id: *mut wire_uint_8_list,
    product_name: *mut wire_uint_8_list,
) {
    wire_add_minor_product_impl(port_, reqeust, major_product_id, product_name)
}

#[no_mangle]
pub extern "C" fn wire_remove_major_product(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    major_product_id: *mut wire_uint_8_list,
) {
    wire_remove_major_product_impl(port_, reqeust, major_product_id)
}

#[no_mangle]
pub extern "C" fn wire_remove_minor_product(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    major_product_id: *mut wire_uint_8_list,
    product_id: *mut wire_uint_8_list,
) {
    wire_remove_minor_product_impl(port_, reqeust, major_product_id, product_id)
}

#[no_mangle]
pub extern "C" fn wire_query_product(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    product_id: *mut wire_uint_8_list,
) {
    wire_query_product_impl(port_, reqeust, product_id)
}

#[no_mangle]
pub extern "C" fn wire_query_all_product(port_: i64, reqeust: *mut wire_ApiRequestCommon) {
    wire_query_all_product_impl(port_, reqeust)
}

#[no_mangle]
pub extern "C" fn wire_update_thing(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    thing_id: *mut wire_uint_8_list,
    thing_name: *mut wire_uint_8_list,
) {
    wire_update_thing_impl(port_, reqeust, thing_id, thing_name)
}

#[no_mangle]
pub extern "C" fn wire_query_all_thing(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    brand_id: *mut wire_uint_8_list,
    product_id: *mut wire_uint_8_list,
) {
    wire_query_all_thing_impl(port_, reqeust, brand_id, product_id)
}

#[no_mangle]
pub extern "C" fn wire_add_group(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    schedule_name: *mut wire_uint_8_list,
    schedule_img_index: *mut u32,
) {
    wire_add_group_impl(port_, reqeust, schedule_name, schedule_img_index)
}

#[no_mangle]
pub extern "C" fn wire_add_maual_schedule(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    schedule_name: *mut wire_uint_8_list,
    schedule_img_index: *mut u32,
) {
    wire_add_maual_schedule_impl(port_, reqeust, schedule_name, schedule_img_index)
}

#[no_mangle]
pub extern "C" fn wire_add_timperiod_schedule(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    schedule_name: *mut wire_uint_8_list,
    schedule_img_index: *mut u32,
) {
    wire_add_timperiod_schedule_impl(port_, reqeust, schedule_name, schedule_img_index)
}

#[no_mangle]
pub extern "C" fn wire_update_schedule_property(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    schedule_id: *mut wire_uint_8_list,
    thing_relation: *mut wire_list___record__String_list___record__String_String,
) {
    wire_update_schedule_property_impl(port_, reqeust, schedule_id, thing_relation)
}

#[no_mangle]
pub extern "C" fn wire_remove_schedule_property(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    schedule_id: *mut wire_uint_8_list,
    thing_relation: *mut wire_StringList,
) {
    wire_remove_schedule_property_impl(port_, reqeust, schedule_id, thing_relation)
}

#[no_mangle]
pub extern "C" fn wire_update_timeperiod_schedule_info(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    schedule_id: *mut wire_uint_8_list,
    hour: u32,
    minute: u32,
    cycle_week_time: u32,
) {
    wire_update_timeperiod_schedule_info_impl(
        port_,
        reqeust,
        schedule_id,
        hour,
        minute,
        cycle_week_time,
    )
}

#[no_mangle]
pub extern "C" fn wire_update_schedule_info(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    schedule_id: *mut wire_uint_8_list,
    schedule_name: *mut wire_uint_8_list,
    schedule_img_index: *mut u32,
) {
    wire_update_schedule_info_impl(
        port_,
        reqeust,
        schedule_id,
        schedule_name,
        schedule_img_index,
    )
}

#[no_mangle]
pub extern "C" fn wire_enable_schedule(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    schedule_id: *mut wire_uint_8_list,
) {
    wire_enable_schedule_impl(port_, reqeust, schedule_id)
}

#[no_mangle]
pub extern "C" fn wire_disable_schedule(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    schedule_id: *mut wire_uint_8_list,
) {
    wire_disable_schedule_impl(port_, reqeust, schedule_id)
}

#[no_mangle]
pub extern "C" fn wire_remove_schedule(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    schedule_id: *mut wire_uint_8_list,
) {
    wire_remove_schedule_impl(port_, reqeust, schedule_id)
}

#[no_mangle]
pub extern "C" fn wire_query_schedule(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    schedule_id: *mut wire_uint_8_list,
) {
    wire_query_schedule_impl(port_, reqeust, schedule_id)
}

#[no_mangle]
pub extern "C" fn wire_query_all_simple_schedule(port_: i64, reqeust: *mut wire_ApiRequestCommon) {
    wire_query_all_simple_schedule_impl(port_, reqeust)
}

#[no_mangle]
pub extern "C" fn wire_execute_schedule(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    schedule_id: *mut wire_uint_8_list,
) {
    wire_execute_schedule_impl(port_, reqeust, schedule_id)
}

#[no_mangle]
pub extern "C" fn wire_hci_search_thing(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    brand_id: *mut wire_uint_8_list,
) {
    wire_hci_search_thing_impl(port_, reqeust, brand_id)
}

#[no_mangle]
pub extern "C" fn wire_hci_add_thing(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    brand_id: *mut wire_uint_8_list,
    major_product_id: *mut wire_uint_8_list,
    minor_product_id: *mut wire_uint_8_list,
    thing_mac: *mut wire_uint_8_list,
    thing_name: *mut wire_uint_8_list,
    thing_data: *mut wire_list___record__String_String,
) {
    wire_hci_add_thing_impl(
        port_,
        reqeust,
        brand_id,
        major_product_id,
        minor_product_id,
        thing_mac,
        thing_name,
        thing_data,
    )
}

#[no_mangle]
pub extern "C" fn wire_hci_remove_thing(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    thing_id: *mut wire_uint_8_list,
) {
    wire_hci_remove_thing_impl(port_, reqeust, thing_id)
}

#[no_mangle]
pub extern "C" fn wire_hci_pair_thing(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    thing_id: *mut wire_uint_8_list,
    operator_data: *mut wire_list___record__String_String,
) {
    wire_hci_pair_thing_impl(port_, reqeust, thing_id, operator_data)
}

#[no_mangle]
pub extern "C" fn wire_hci_removepair_thing(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    thing_id: *mut wire_uint_8_list,
    operator_data: *mut wire_list___record__String_String,
) {
    wire_hci_removepair_thing_impl(port_, reqeust, thing_id, operator_data)
}

#[no_mangle]
pub extern "C" fn wire_hci_query_thing(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    thing_id: *mut wire_uint_8_list,
) {
    wire_hci_query_thing_impl(port_, reqeust, thing_id)
}

#[no_mangle]
pub extern "C" fn wire_hci_ctrl_thing(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    thing_id: *mut wire_uint_8_list,
    operator_data: *mut wire_list___record__String_String,
) {
    wire_hci_ctrl_thing_impl(port_, reqeust, thing_id, operator_data)
}

#[no_mangle]
pub extern "C" fn wire_hci_get_task_result(
    port_: i64,
    reqeust: *mut wire_ApiRequestCommon,
    task_id: u32,
    thing_ids: *mut wire_StringList,
) {
    wire_hci_get_task_result_impl(port_, reqeust, task_id, thing_ids)
}

// Section: allocate functions

#[no_mangle]
pub extern "C" fn new_StringList_0(len: i32) -> *mut wire_StringList {
    let wrap = wire_StringList {
        ptr: support::new_leak_vec_ptr(<*mut wire_uint_8_list>::new_with_null_ptr(), len),
        len,
    };
    support::new_leak_box_ptr(wrap)
}

#[no_mangle]
pub extern "C" fn new_box_autoadd_api_request_common_0() -> *mut wire_ApiRequestCommon {
    support::new_leak_box_ptr(wire_ApiRequestCommon::new_with_null_ptr())
}

#[no_mangle]
pub extern "C" fn new_box_autoadd_u32_0(value: u32) -> *mut u32 {
    support::new_leak_box_ptr(value)
}

#[no_mangle]
pub extern "C" fn new_list___record__String_String_0(
    len: i32,
) -> *mut wire_list___record__String_String {
    let wrap = wire_list___record__String_String {
        ptr: support::new_leak_vec_ptr(<wire___record__String_String>::new_with_null_ptr(), len),
        len,
    };
    support::new_leak_box_ptr(wrap)
}

#[no_mangle]
pub extern "C" fn new_list___record__String_list___record__String_String_0(
    len: i32,
) -> *mut wire_list___record__String_list___record__String_String {
    let wrap = wire_list___record__String_list___record__String_String {
        ptr: support::new_leak_vec_ptr(
            <wire___record__String_list___record__String_String>::new_with_null_ptr(),
            len,
        ),
        len,
    };
    support::new_leak_box_ptr(wrap)
}

#[no_mangle]
pub extern "C" fn new_uint_8_list_0(len: i32) -> *mut wire_uint_8_list {
    let ans = wire_uint_8_list {
        ptr: support::new_leak_vec_ptr(Default::default(), len),
        len,
    };
    support::new_leak_box_ptr(ans)
}

// Section: related functions

// Section: impl Wire2Api

impl Wire2Api<String> for *mut wire_uint_8_list {
    fn wire2api(self) -> String {
        let vec: Vec<u8> = self.wire2api();
        String::from_utf8_lossy(&vec).into_owned()
    }
}
impl Wire2Api<Vec<String>> for *mut wire_StringList {
    fn wire2api(self) -> Vec<String> {
        let vec = unsafe {
            let wrap = support::box_from_leak_ptr(self);
            support::vec_from_leak_ptr(wrap.ptr, wrap.len)
        };
        vec.into_iter().map(Wire2Api::wire2api).collect()
    }
}
impl Wire2Api<(String, String)> for wire___record__String_String {
    fn wire2api(self) -> (String, String) {
        (self.field0.wire2api(), self.field1.wire2api())
    }
}
impl Wire2Api<(String, Vec<(String, String)>)>
    for wire___record__String_list___record__String_String
{
    fn wire2api(self) -> (String, Vec<(String, String)>) {
        (self.field0.wire2api(), self.field1.wire2api())
    }
}
impl Wire2Api<ApiRequestCommon> for wire_ApiRequestCommon {
    fn wire2api(self) -> ApiRequestCommon {
        ApiRequestCommon {
            target: self.target.wire2api(),
        }
    }
}
impl Wire2Api<ApiRequestCommon> for *mut wire_ApiRequestCommon {
    fn wire2api(self) -> ApiRequestCommon {
        let wrap = unsafe { support::box_from_leak_ptr(self) };
        Wire2Api::<ApiRequestCommon>::wire2api(*wrap).into()
    }
}
impl Wire2Api<u32> for *mut u32 {
    fn wire2api(self) -> u32 {
        unsafe { *support::box_from_leak_ptr(self) }
    }
}
impl Wire2Api<Vec<(String, String)>> for *mut wire_list___record__String_String {
    fn wire2api(self) -> Vec<(String, String)> {
        let vec = unsafe {
            let wrap = support::box_from_leak_ptr(self);
            support::vec_from_leak_ptr(wrap.ptr, wrap.len)
        };
        vec.into_iter().map(Wire2Api::wire2api).collect()
    }
}
impl Wire2Api<Vec<(String, Vec<(String, String)>)>>
    for *mut wire_list___record__String_list___record__String_String
{
    fn wire2api(self) -> Vec<(String, Vec<(String, String)>)> {
        let vec = unsafe {
            let wrap = support::box_from_leak_ptr(self);
            support::vec_from_leak_ptr(wrap.ptr, wrap.len)
        };
        vec.into_iter().map(Wire2Api::wire2api).collect()
    }
}

impl Wire2Api<Vec<u8>> for *mut wire_uint_8_list {
    fn wire2api(self) -> Vec<u8> {
        unsafe {
            let wrap = support::box_from_leak_ptr(self);
            support::vec_from_leak_ptr(wrap.ptr, wrap.len)
        }
    }
}
// Section: wire structs

#[repr(C)]
#[derive(Clone)]
pub struct wire_StringList {
    ptr: *mut *mut wire_uint_8_list,
    len: i32,
}

#[repr(C)]
#[derive(Clone)]
pub struct wire___record__String_String {
    field0: *mut wire_uint_8_list,
    field1: *mut wire_uint_8_list,
}

#[repr(C)]
#[derive(Clone)]
pub struct wire___record__String_list___record__String_String {
    field0: *mut wire_uint_8_list,
    field1: *mut wire_list___record__String_String,
}

#[repr(C)]
#[derive(Clone)]
pub struct wire_ApiRequestCommon {
    target: *mut wire_uint_8_list,
}

#[repr(C)]
#[derive(Clone)]
pub struct wire_list___record__String_String {
    ptr: *mut wire___record__String_String,
    len: i32,
}

#[repr(C)]
#[derive(Clone)]
pub struct wire_list___record__String_list___record__String_String {
    ptr: *mut wire___record__String_list___record__String_String,
    len: i32,
}

#[repr(C)]
#[derive(Clone)]
pub struct wire_uint_8_list {
    ptr: *mut u8,
    len: i32,
}

// Section: impl NewWithNullPtr

pub trait NewWithNullPtr {
    fn new_with_null_ptr() -> Self;
}

impl<T> NewWithNullPtr for *mut T {
    fn new_with_null_ptr() -> Self {
        std::ptr::null_mut()
    }
}

impl NewWithNullPtr for wire___record__String_String {
    fn new_with_null_ptr() -> Self {
        Self {
            field0: core::ptr::null_mut(),
            field1: core::ptr::null_mut(),
        }
    }
}

impl Default for wire___record__String_String {
    fn default() -> Self {
        Self::new_with_null_ptr()
    }
}

impl NewWithNullPtr for wire___record__String_list___record__String_String {
    fn new_with_null_ptr() -> Self {
        Self {
            field0: core::ptr::null_mut(),
            field1: core::ptr::null_mut(),
        }
    }
}

impl Default for wire___record__String_list___record__String_String {
    fn default() -> Self {
        Self::new_with_null_ptr()
    }
}

impl NewWithNullPtr for wire_ApiRequestCommon {
    fn new_with_null_ptr() -> Self {
        Self {
            target: core::ptr::null_mut(),
        }
    }
}

impl Default for wire_ApiRequestCommon {
    fn default() -> Self {
        Self::new_with_null_ptr()
    }
}

// Section: sync execution mode utility

#[no_mangle]
pub extern "C" fn free_WireSyncReturn(ptr: support::WireSyncReturn) {
    unsafe {
        let _ = support::box_from_leak_ptr(ptr);
    };
}
