
use std::{os::raw::c_char, ffi::{CString, CStr}, };

#[no_mangle]
pub unsafe extern "C" fn bm_get_error() -> *const c_char {
    let error_string = crate::get_last_error();

    CString::new(error_string.as_str())
        .unwrap_or(CString::new("").unwrap())
        .into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn bm_init(people: *const c_char, core: *const c_char) -> bool {

    let people = unsafe { CStr::from_ptr(people) }.to_string_lossy().to_string();
    let core = unsafe { CStr::from_ptr(core) }.to_string_lossy().to_string();

    crate::init(people, core).is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn bm_add_brand(brand_name: *const c_char) -> Vec<u8> {
    let brand_name = unsafe { CStr::from_ptr(brand_name) }.to_string_lossy().to_string();

    let r = crate::add_brand(brand_name);
}

// #[no_mangle]
// pub unsafe extern "C" fn bm_test_brand() -> *const crate::api::TestBrand {
//     let brand = std::sync::Arc::new({
//         let mut brand = crate::api::TestBrand {
//             brand_id: "1001".to_string(),
//             brand_name: "test10001".to_string(),
//             begin_time: "2023-06-01".to_string(),
//             update_time: "2023-06-01".to_string(),
//             status: 1,
        
//         };
//         brand
//     });

//     std::sync::Arc::as_ptr(&brand)
//     // let brand_name = unsafe { CStr::from_ptr(brand_name) }.to_string_lossy().to_string();

//     // crate::add_brand(brand_name).is_ok()
// }
