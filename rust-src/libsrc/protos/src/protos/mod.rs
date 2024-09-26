// @by near generated
use crate::inner_impl_default_protobuf_raw_codec;

pub mod proof;
pub mod hci;
pub mod cfg;
pub mod core_message;
inner_impl_default_protobuf_raw_codec!(core_message::Message);
inner_impl_default_protobuf_raw_codec!(core_message::Subscribe_message);
inner_impl_default_protobuf_raw_codec!(core_message::Dissubscribe_message);
inner_impl_default_protobuf_raw_codec!(core_message::Dispatch_message);
pub mod profile;
inner_impl_default_protobuf_raw_codec!(profile::Data);
