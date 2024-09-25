
use near_base::{NearError, NearResult};

pub enum DataContent<T> {
    Error(NearError),
    Content(T),
}

impl<T> From<NearResult<T>> for DataContent<T> {
    fn from(value: NearResult<T>) -> Self {
        match value {
            Ok(v) => Self::Content(v),
            Err(e) => Self::Error(e)
        }
    }
}

#[allow(non_snake_case)]
pub mod ProtobufObjectHelper {

    use base::{raw_object::{RawObjectBuilder, RawObjectDescContent, RawObjectBodyContent, RawObjectGuard, RawContent, }};
    use log::error;
    use near_base::{NearResult, NearError, ErrorCode};
    use prost::Message;
    use prost_reflect::DynamicMessage;

    const PROTOBUF_RAW_OBJECT_VERSION: u8 = 1u8;

    use crate::{RAW_OBJECT_FORMAT, protoc_utils::utils::create_message, Empty};

    use super::DataContent;

    pub fn encode(value: DataContent<impl protobuf::Message>) -> NearResult<RawObjectGuard> {

        match value {
            DataContent::Error(e) => encode_with_errno(e),
            DataContent::Content(c) => encode_with_message(c),
        }

    }

    pub fn encode_none() -> NearResult<RawObjectGuard> {
        encode_with_message(Empty::new())
    }

    pub fn encode_with_errno(e: NearError) -> NearResult<RawObjectGuard> {
        RawObjectBuilder::new(RawObjectDescContent::default()
                                        .set_version(PROTOBUF_RAW_OBJECT_VERSION)
                                        .set_with_error(e),
                              RawObjectBodyContent{})
            .build()
            .map(| o | o.into())
            .map_err(| err | {
                error!("Failed build raw-object with {}", err);
                err
            })
    }

    pub fn encode_with_message(value: impl protobuf::Message) -> NearResult<RawObjectGuard> {
        let text =
            value.write_to_bytes()
                .map_err(| err | {
                    let error_string = format!("Failed serialize message with err = {}", err);
                    error!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_PROTOC_ENCODE, error_string)
                })?;

        RawObjectBuilder::new(RawObjectDescContent::default()
                                        .set_version(PROTOBUF_RAW_OBJECT_VERSION)
                                        .set_with_data(RAW_OBJECT_FORMAT::Protobuf as u8, text),
                              RawObjectBodyContent{})
        .build()
        .map(| o | o.into())
        .map_err(| err | {
            error!("Failed build raw-object with {}", err);
            err
        })
    }

    pub fn encode_with_dynamic_message(value: prost_reflect::DynamicMessage) -> NearResult<RawObjectGuard> {
        let text = {
            let size = value.encoded_len();
            let mut text = vec![0u8; size];

            match value.encode(&mut text) {
                Ok(_) => { Ok(text) }
                Err(err) => {
                    let error_string = format!("Failed serialize dynamic-message with err = {}", err);
                    error!("{error_string}");
                    Err(NearError::new(ErrorCode::NEAR_ERROR_PROTOC_ENCODE, error_string))
                }
            }
        }?;

        RawObjectBuilder::new(RawObjectDescContent::default()
                                    .set_version(PROTOBUF_RAW_OBJECT_VERSION)
                                    .set_with_data(RAW_OBJECT_FORMAT::Protobuf as u8, text),
                              RawObjectBodyContent{})
            .build()
            .map(| o | o.into())
            .map_err(| err | {
                error!("Failed build raw-object with {}", err);
                err
            })

    }

    pub fn decode_with_name(raw_object: RawObjectGuard,
                            message_name: &str) -> NearResult<DataContent<prost_reflect::DynamicMessage>> {
        match raw_object.desc().content().data() {
            RawContent::Error(e) => Ok(DataContent::Error(e.clone())),
            RawContent::Content(c) => {
                if c.format != RAW_OBJECT_FORMAT::Protobuf as u8 {
                    let error_string = format!("Failed decode raw-object because itn't protobuf, except={}", c.format);
                    error!("{error_string}");
                    return Err(NearError::new(ErrorCode::NEAR_ERROR_PROTOC_DECODE, error_string));
                }

                let dynamic_message =
                    DynamicMessage::decode(create_message(message_name)
                                            .map_err(| err | {
                                                error!("Failed create_message with err {}", err);
                                                err
                                            })?,
                                            c.data.as_slice())
                        .map_err(| err | {
                            let error_string = format!("Failed deserialize dynamic-message with err = {}", err);
                            error!("{error_string}");
                            NearError::new(ErrorCode::NEAR_ERROR_PROTOC_DECODE, error_string)
                        })?;

                Ok(DataContent::Content(dynamic_message))
            }
            _ => unreachable!(),
        }
    }

    pub fn decode<T>(raw_object: RawObjectGuard) -> NearResult<DataContent<T>>
    where T: protobuf::Message {

        match raw_object.desc().content().data() {
            RawContent::Error(e) => Ok(DataContent::Error(e.clone())),
            RawContent::Content(c) => {
                if c.format != RAW_OBJECT_FORMAT::Protobuf as u8 {
                    let error_string = format!("Failed decode raw-object because itn't protobuf, except={}", c.format);
                    error!("{error_string}");
                    return Err(NearError::new(ErrorCode::NEAR_ERROR_PROTOC_DECODE, error_string));
                }

                let mut value = T::default();

                value.merge_from_bytes(c.data.as_slice())
                    .map_err(| err | {
                        let error_string = format!("Failed decode raw-object with err = {}", err);
                        error!("{error_string}");
                        NearError::new(ErrorCode::NEAR_ERROR_PROTOC_DECODE, error_string)
                    })?;

                Ok(DataContent::Content(value))

            }
            _ => unreachable!()
        }
    }
}
