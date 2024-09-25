
use std::io::Read;

use log::error;

use near_base::{NearResult, ErrorCode, NearError};

use crate::get_descriptor_bin;

#[allow(unused)]
pub fn init_protos_manager() -> NearResult<()> {
    let mut text = vec![];
    std::fs::File::open(get_descriptor_bin())
        .map_err(| _ | {
            let error_string = format!("failed read file with err = {}", stringify!(err));
            error!("{error_string}");
            NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string)
        })?
        .read_to_end(&mut text)
        .map_err(| _ | {
            let error_string = format!("failed read file with err = {}", stringify!(err));
            error!("{error_string}");
            NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, error_string)
        })?;

    // let mut pool = prost_reflect::DescriptorPool::global();
    prost_reflect::DescriptorPool::decode_global_file_descriptor_set(text.as_slice())
    // prost_reflect::DescriptorPool::decode_global_file_descriptor_set(bin_bytes.as_bytes())
        .map_err(| err | {
            let error_string = format!("failed decode descriptor-bin with err = {err}");
            error!("{error_string}");
            NearError::new(ErrorCode::NEAR_ERROR_PROTOC_DECODE, error_string)
        })?;

    Ok(())
}

#[allow(unused)]
pub(crate) fn create_message(class_name: &str) -> NearResult<prost_reflect::MessageDescriptor> {

    let descriptor = prost_reflect::DescriptorPool::global().get_message_by_name(class_name).ok_or_else(|| {
        let error_message = format!("Not found {class_name} Protoc-Message.");
        error!("{error_message}");
        NearError::new(ErrorCode::NEAR_ERROR_PROTOC_NOT_MESSAGE, error_message)
    })?;

    Ok(descriptor)
}

#[allow(unused)]
pub(crate) fn set_message_field(dynamic_message: &mut prost_reflect::DynamicMessage,
                                field_name: &str,
                                val: prost_reflect::Value) -> NearResult<()> {

    dynamic_message.try_set_field_by_name(field_name, val)
        .map_err(| err | {
            let error_string = format!("Failed set field with err = {}", err);
            error!("{error_string}");

            match err {
                prost_reflect::SetFieldError::NotFound => {
                    NearError::new(ErrorCode::NEAR_ERROR_PROTOC_NOT_FIELD, error_string)
                }
                _ => {
                    NearError::new(ErrorCode::NEAR_ERROR_PROTOC_SET_FIELD, error_string)
                }
            }
        })

}

#[test]
fn test_create_message() {
    init_protos_manager().unwrap();

    use prost_reflect::Value;
    use prost_reflect::DynamicMessage;
    use prost::Message;
    use protobuf::Message;
    use bytes::*;

    let text = {
        let mut brand = crate::brand::Brand_info::new();
        // brand.set_brand_id(1);
        brand.set_brand_name("test".to_owned());
        // brand.set_status(0);
        brand.set_begin_time("2022-01-01".to_owned());

        brand.write_to_bytes().unwrap()

        // DynamicMessage::new(brand.descriptor_dyn())
    };

    {
        let message_descriptor = create_message("brand_info").unwrap();

        {
            let message = DynamicMessage::decode(message_descriptor, text.as_slice()).unwrap();
            println!("{}", message);
        }
    }

    println!("{}:[{:?}]", text.len(), text);

    let message_descriptor = create_message("brand_info").unwrap();
    let mut dynamic_message = DynamicMessage::new(message_descriptor);

    {
        set_message_field(&mut dynamic_message, "brand_name", Value::String("xxyyzz".to_owned())).unwrap();
        set_message_field(&mut dynamic_message, "brand_id", Value::U32(2)).unwrap();
        set_message_field(&mut dynamic_message, "status", Value::U32(1)).unwrap();
        set_message_field(&mut dynamic_message, "begin_time", Value::String("2023-12-31".to_owned())).unwrap();
        // let m = DynamicMessage::decode(descriptor, text.as_slice()).unwrap();

        // println!("{}", m);
    }
    {
        // let dynamic_message = descriptor.options();
        let data = dynamic_message.encode_to_vec();
    
        println!("{}:[{:?}]", data.len(), data);

        let mut d_brand = crate::brand::Brand_info::new();
        d_brand.merge_from_bytes(&data).unwrap();

        println!("{}", d_brand);
    }


}
