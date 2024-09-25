
pub mod codec;
pub mod components;
pub mod errors;
pub mod time;
pub mod waiter;
pub mod crypto_module;
pub mod dynamic_ptr;
pub mod queue;

pub use errors::*;
pub use time::*;
pub use waiter::*;
pub use codec::*;
pub use components::*;
pub use crypto_module::*;
pub use dynamic_ptr::*;

pub trait ToNearError {
    fn to_near_error(self) -> NearError;
}

pub mod utils {

    #[inline]
    #[allow(dead_code)]
    pub fn make_word(h: u8, l: u8) -> u16 {
        ((h as u16) << 8) | (l as u16)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn unmake_word(v: u16) -> (u8, u8) {
        let h = (v >> 8) as u8;
        let l = ((v << 8) >> 8) as u8;
        (h, l)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn make_long(h: u16, l: u16) -> u32 {
        ((h as u32) << 16) | (l as u32)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn unmake_long(v: u32) -> (u16, u16) {
        let h = (v >> 16) as u16;
        let l = ((v << 16) >> 16) as u16;
        (h, l)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn make_longlong(h: u32, l: u32) -> u64 {
        ((h as u64) << 32 as u64) | (l as u64)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn unmake_longlong(v: u64) -> (u32, u32) {
        let h = (v >> 32) as u32;
        let l = ((v << 32) >> 32) as u32;
        (h, l)
    }

}

#[macro_export]
macro_rules! app_err {
    ( $err: expr) => {
        // errors::NearError::new(errors::ErrorCode::)
        // near_base::NearError::new(near_base::ErrorCo)
    // cyfs_base::BuckyError::new(cyfs_base::BuckyErrorCodeEx::DecError($err as u16), format!("{}:{} app_code_err:{}", file!(), line!(), stringify!($err)))
    };
}

#[macro_export]
macro_rules! println_err {
    ( $msg: expr ) => {
        // println()
        // errors::NearError::new($err, format!("{}:{} "))
    // cyfs_base::BuckyError::new(cyfs_base::BuckyErrorCodeEx::DecError($err as u16), format!("{}:{} app_code_err:{} msg:{}", file!(), line!(), stringify!($err), $msg))
    };
}
