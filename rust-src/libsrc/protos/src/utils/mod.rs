
pub mod raw_utils;
pub mod protoc_utils;

use near_base::{NearResult, NearError};

const RAW_OBJECT_VERSION: u8 = 1u8;

pub enum DataContent<T> {
    Error(NearError),
    Content(T),
}

impl<T> From<DataContent<T>> for NearResult<T> {
    fn from(value: DataContent<T>) -> Self {
        match value {
            DataContent::Content(v) => Ok(v),
            DataContent::Error(e) => Err(e),
        }
    }
}

impl<T> From<NearResult<T>> for DataContent<T> {
    fn from(value: NearResult<T>) -> Self {
        match value {
            Ok(v) => Self::Content(v),
            Err(e) => Self::Error(e)
        }
    }
}

impl<T> From<T> for DataContent<T> {
    fn from(value: T) -> Self {
        Self::Content(value)
    }
}

#[macro_export]
macro_rules! try_encode_raw_object {
    ($rr: ident, $label: tt) => {
        match protos::RawObjectHelper::encode($rr) {
            Ok(raw_object) => {
                log::info!("encode-data:{}, sequence: {}", raw_object, $label);
                EventResult::Response(raw_object.into())
            }
            Err(e) => {
                let error_string = format!("failed encode to message with err: {e}");
                log::error!("{error_string}, sequence: {}", $label);
                EventResult::Ignore
            }
        }
    };
}

#[macro_export]
macro_rules! try_decode_raw_object {
    ($T: ty, $rr: ident, $body: ident, $req_body: tt, $label: tt) => {
        match protos::RawObjectHelper::decode::<$T>($rr) {
            Ok(data) => {
                match data {
                    protos::DataContent::Content(mut $body) => {
                        log::debug!("decode-data: {:?}, sequence: {}", $body, $label);
                        let r = $req_body; DataContent::Content(r)
                    },
                    protos::DataContent::Error(_) => unreachable!()
                }
            }
            Err(e) => {
                let error_string = format!("failed decode message with err: {e}");
                log::error!("{error_string} sequence: {}", $label);
                protos::DataContent::Error(e)
            }
        }
    };
}

