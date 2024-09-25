
mod protos;
mod utils;

use std::path::PathBuf;

pub use protos::*;
pub use utils::{raw_utils::helper::RawObjectHelper,
                DataContent,
                protoc_utils
            };

#[repr(u8)]
pub(crate) enum RawObjectFormat {
    #[allow(unused)]
    Raw = 1u8,
    #[allow(unused)]
    Protobuf = 2u8,
    #[allow(unused)]
    Json = 3u8,
}

const DESCRIPTOR_BIN: &'static str = "near_descriptor_set.bin";

lazy_static::lazy_static! {
    static ref NEAR_DESCRIPTOR_BIN: PathBuf = near_core::get_data_path().join("bin").join(DESCRIPTOR_BIN);
}

#[allow(unused)]
pub(crate) fn get_descriptor_bin() -> PathBuf {
    NEAR_DESCRIPTOR_BIN.clone()
}

#[macro_export(local_inner_macros)]
macro_rules! inner_impl_default_protobuf_raw_codec {
    ($proto_name:ty) => {
        impl near_base::Serialize for $proto_name {
            fn raw_capacity(&self) -> usize {
                crate::utils::raw_utils::helper::ProtobufObjectCodecHelper::raw_capacity(self)
            }

            fn serialize<'a>(&self,
                             buf: &'a mut [u8]) -> near_base::NearResult<&'a mut [u8]> {
                crate::utils::raw_utils::helper::ProtobufObjectCodecHelper::serialize(self, buf)
            }
        }

        impl near_base::Deserialize for $proto_name {
            fn deserialize<'de>(buf: &'de [u8]) -> near_base::NearResult<(Self, &'de [u8])> {
                crate::utils::raw_utils::helper::ProtobufObjectCodecHelper::deserialize(buf)
            }
        }
    }
}
