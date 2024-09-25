
#[allow(non_snake_case)]
pub mod ProtobufObjectCodecHelper {

    use near_base::{NearResult, NearError, ErrorCode};

    pub fn raw_capacity(v: &impl protobuf::Message) -> usize {
        v.compute_size() as usize
    }

    pub fn serialize<'a>(v: &impl protobuf::Message,
                         buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {

        let size = v.compute_size() as usize;
        let buf_size = buf.len();

        if size > buf_size {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, format!("not enough buffer for serialize protobuf message, except={}, got={}", size, buf_size)));
        }

        let mut stream = ::protobuf::CodedOutputStream::bytes(buf);

           v.write_to(&mut stream)
            .map_err(|e| {
                NearError::new(ErrorCode::NEAR_ERROR_3RD, format!("failed encode protobuf::Message to stream with err = {}", e))
            })?;
        drop(stream);

        Ok(&mut buf[size..])

    }

    pub fn deserialize<'de, T>(buf: &'de [u8]) -> NearResult<(T, &'de [u8])> 
    where T: protobuf::Message {
        let size = buf.len();

        // 必须截取精确大小的buffer
        let mut stream = ::protobuf::CodedInputStream::from_bytes(buf);
        let v = T::parse_from(&mut stream)
                    .map_err(|e| {
            NearError::new(ErrorCode::NEAR_ERROR_3RD, format!("failed decode protobuf::Message from stream with err = {}", e))
        })?;

        assert_eq!(stream.pos() as usize, size);

        Ok((v, &buf[size..]))

    }

}

#[allow(non_snake_case)]
pub mod RawObjectHelper {

    use log::error;
    use base::raw_object::{RawObjectBuilder, RawObjectDescContent, RawObjectBodyContent, RawObjectGuard, RawContent};
    use near_base::{NearResult, Serialize, NearError, builder_codec_macro::Empty, Deserialize, ErrorCode, };
    use crate::{RawObjectFormat, utils::RAW_OBJECT_VERSION, DataContent};

    pub fn encode<T: Serialize>(data: DataContent<T>) -> NearResult<RawObjectGuard> {
        match data {
            DataContent::Content(v) => encode_with_raw(v),
            DataContent::Error(e) => encode_with_error(e)
        }
    }

    pub fn encode_none() -> NearResult<RawObjectGuard> {
        encode_with_raw(Empty)
    }

    pub fn encode_with_error(e: NearError) -> NearResult<RawObjectGuard> {
        RawObjectBuilder::new(RawObjectDescContent::default()
                                    .set_version(RAW_OBJECT_VERSION)
                                    .set_with_error(e),
                            RawObjectBodyContent{})
            .build()
            .map(| o | o.into())
            .map_err(| err | {
                error!("Failed build raw-object with {}", err);
                err
            })
    }

    pub fn encode_with_raw<T: Serialize>(raw_data: T) -> NearResult<RawObjectGuard> {

        let text = {
            let mut text = vec![0u8; raw_data.raw_capacity()];
            let _ = raw_data.serialize(&mut text)?;
            text
        };

        RawObjectBuilder::new(
            RawObjectDescContent::default()
                    .set_version(RAW_OBJECT_VERSION)
                    .set_with_data(RawObjectFormat::Raw as u8, text),
            RawObjectBodyContent{}
        )
        .build()
        .map(| o | o.into())
        .map_err(| err | {
            error!("Failed build raw-object with {}", err);
            err
        })

    }

    #[cfg(feature = "use_dynamic_message")]
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
                                    .set_version(RAW_OBJECT_VERSION)
                                    .set_with_data(RawObjectFormat::Protobuf as u8, text),
                              RawObjectBodyContent{})
            .build()
            .map(| o | o.into())
            .map_err(| err | {
                error!("Failed build raw-object with {}", err);
                err
            })

    }

    #[cfg(feature = "use_dynamic_message")]
    pub fn decode_with_name(raw_object: RawObjectGuard,
                            message_name: &str) -> NearResult<DataContent<prost_reflect::DynamicMessage>> {
        match raw_object.desc().content().data() {
            RawContent::Error(e) => Ok(DataContent::Error(e.clone())),
            RawContent::Content(c) => {
                if c.format != RawObjectFormat::Protobuf as u8 {
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
    where T: Serialize + Deserialize {
        match raw_object.desc().content().data() {
            RawContent::Error(e) => Ok(DataContent::Error(e.clone())),
            RawContent::Content(c) => {
                if c.format != RawObjectFormat::Raw as u8 {
                    let error_string = format!("Failed decode raw-object because itn't raw data, except={}", c.format);
                    error!("{error_string}");
                    return Err(NearError::new(ErrorCode::NEAR_ERROR_PROTOC_DECODE, error_string));
                }

                // let mut value = T::default();
                let (r, _ ) = T::deserialize(c.data.as_slice())
                                    .map_err(| err | {
                                        let error_string = format!("Failed decode raw-object with err = {}", err);
                                        error!("{error_string}");
                                        NearError::new(ErrorCode::NEAR_ERROR_PROTOC_DECODE, error_string)
                                    })?;

                Ok(DataContent::Content(r))
            }
            _ => unreachable!()
        }
    }
}

#[allow(unused)]
mod test {

    #[test]
    pub fn test_encode_decode() {
        use std::{collections::HashMap, hash::Hash};
        use base::raw_object::RawContent;

        use crate::{protos, RawObjectHelper, DataContent};

        let mut v = vec![];
        let mut h = HashMap::new();

        h.insert("a".to_owned(), "a".to_owned());
        h.insert("b".to_owned(), "b".to_owned());
        h.insert("c".to_owned(), "c".to_owned());
        h.insert("d".to_owned(), "d".to_owned());
        h.insert("e".to_owned(), "e".to_owned());
        
        for n in vec!["123".to_owned(), "234".to_owned(), "345".to_owned()] {
            v.push((
                n,
                h.clone()
            ))
        }

        let r = RawObjectHelper::encode_with_raw(v).unwrap();

        if let RawContent::Content(raw) = r.desc().content().data() {
            println!("{:?}", raw.data);
        }

        {
            let xx = RawObjectHelper::decode::<Vec<(String, HashMap<String, String>)>>(r).unwrap();

            if let DataContent::Content(c) = xx {
                println!("{:?}", c);
            }
        }
    }
}
