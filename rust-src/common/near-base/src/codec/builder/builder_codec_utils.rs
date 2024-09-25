
use std::{path::Path, io::Read};

use crate::*;

pub trait FileEncoder {
    fn encode_to_writer(&self, w: impl std::io::Write, is_compress: bool) -> NearResult<usize>;

    fn encode_to_file(&self, file: &Path, is_compress: bool) -> NearResult<usize> {
        match std::fs::File::create(file) {
            Ok(file) => self.encode_to_writer(file, is_compress),
            Err(e) => Err(NearError::from(e)),
        }
    }
}

impl<D> FileEncoder for D
where D: Serialize {
    fn encode_to_writer(&self, mut w: impl std::io::Write, _is_compress: bool) -> NearResult<usize> {
        let len = self.raw_capacity();
        let mut buf: Vec<u8> = vec![0u8; len];
        let _ = self.serialize(buf.as_mut_slice())?;

        let len = w.write(buf.as_slice())
                          .map_err(| err | NearError::from(err) )?;
        Ok(len)
    }
}

pub trait FileDecoder: Sized {
    fn decode_from_file<'de>(file: &Path) -> NearResult<Self>;
}

impl<D> FileDecoder for D
where D: Deserialize, {
    fn decode_from_file<'de>(file: &Path) -> NearResult<Self> {
        match std::fs::File::open(file) {
            Ok(mut file) => {
                let mut data = vec![];
                if let Err(err) = file.read_to_end(&mut data) {
                    Err(NearError::from(err))
                } else {
                    let (obj, _) = D::deserialize(data.as_slice())?;
                    Ok(obj)
                }
            }
            Err(err) => Err(NearError::from(err))
        }
    }
}

pub trait RawConvertTo<O> {
    fn to_vec(&self) -> NearResult<Vec<u8>>;
    fn to_hex(&self) -> NearResult<String>;
}

// pub trait RawFrom<'de, O> {
//     fn clone_from_slice(buf: &'de [u8]) -> NearResult<O>;
//     fn clone_from_hex(hex_str: &str, buf: &'de mut Vec<u8>) -> NearResult<O>;
// }

impl<T> RawConvertTo<T> for T
where
    T: Serialize,
{
    fn to_vec(&self) -> NearResult<Vec<u8>> {
        let mut data = vec![0u8; self.raw_capacity()];

        let _ = self.serialize(&mut data)?;

        Ok(data)
    }

    fn to_hex(&self) -> NearResult<String> {
        let buf = self.to_vec()?;
        Ok(hex::encode(buf))
    }
}

// impl<'de, O> RawFrom<'de, O> for O
// where
//     O: RawDecode<'de>,
// {
//     fn clone_from_slice(buf: &'de [u8]) -> BuckyResult<O> {
//         let (t, _buf) = O::raw_decode(buf)?;

//         // println!("buffer_len:{}", buf.len());
//         // assert_eq!(_buf.len(),0);
//         Ok(t)
//     }

//     fn clone_from_hex(hex_str: &str, buf: &'de mut Vec<u8>) -> BuckyResult<O> {
//         let buf_size = hex_str.len() / 2;
//         buf.resize(buf_size, 0);
//         hex::decode_to_slice(hex_str, buf)?;

//         let (t, _buf) = O::raw_decode(buf)?;

//         Ok(t)
//     }
// }
