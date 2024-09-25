
use generic_array::{GenericArray, ArrayLength};

use std::collections::{BTreeMap, LinkedList, HashMap};
use std::vec;

use crate::codec::builder_codec::{BuilderCounter, SERIALIZE_HEADER_SIZE, Serialize, Deserialize};
use crate::errors::{NearResult, NearError, ErrorCode};

const U8_CAPACITY: usize    = std::mem::size_of::<u8>();
const U16_CAPACITY: usize   = std::mem::size_of::<u16>();
const U32_CAPACITY: usize   = std::mem::size_of::<u32>();
const U64_CAPACITY: usize   = std::mem::size_of::<u64>();
const U128_CAPACITY: usize  = std::mem::size_of::<u128>();
const I8_CAPACITY: usize    = std::mem::size_of::<i8>();
const I16_CAPACITY: usize   = std::mem::size_of::<i16>();
const I32_CAPACITY: usize   = std::mem::size_of::<i32>();
const I64_CAPACITY: usize   = std::mem::size_of::<i64>();
const I128_CAPACITY: usize  = std::mem::size_of::<i128>();
const F32_CAPACITY: usize   = std::mem::size_of::<f32>();
const F64_CAPACITY: usize   = std::mem::size_of::<f64>();
const BOOL_CAPACITY: usize  = std::mem::size_of::<bool>();

pub fn deserialize_head<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(&'de [u8] /* remain buf */, usize /* capacity */)> {
    let (end, capacity) = {
        if buf.len() < SERIALIZE_HEADER_SIZE {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, 
                                        format!("not enough buffer, except={}, get={}", buf.len(), SERIALIZE_HEADER_SIZE)));
        }

        if builder.next() != buf[0] {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                        format!("packet target({}!={}) invalid", buf[0], builder.curr())));
        }

        (&buf[SERIALIZE_HEADER_SIZE..], buf[1] as usize)
    };

    if end.len() < capacity {
        return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, 
                                    format!("not enough buffer, except={}, get={}", end.len(), capacity)));
    }

    Ok((end, capacity))
}

impl Serialize for usize {
    fn raw_capacity(&self) -> usize {
        let v = *self as u128;

        if v % u8::MAX as u128 == 0 {
            U8_CAPACITY
        } else if v % u16::MAX as u128 == 0 {
            U16_CAPACITY
        } else if v % u32::MAX as u128 == 0 {
            U32_CAPACITY
        } else if v % u64::MAX as u128 == 0 {
            U64_CAPACITY
        } else {
            U128_CAPACITY
        }
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        match self.raw_capacity() {
            U8_CAPACITY => (*self as u8).serialize(buf, builder),
            U16_CAPACITY => (*self as u16).serialize(buf, builder),
            U32_CAPACITY => (*self as u32).serialize(buf, builder),
            U64_CAPACITY => (*self as u64).serialize(buf, builder),
            _ => (*self as u128).serialize(buf, builder),
        }
    }
}

impl Deserialize for usize {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        if buf.len() < SERIALIZE_HEADER_SIZE {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
        }

        let capacity = buf[1] as usize;

        match capacity {
            U8_CAPACITY => {
                let (v, buf) = u8::deserialize(buf, builder)?;
                Ok((v as usize, buf))
            }
            U16_CAPACITY => {
                let (v, buf) = u16::deserialize(buf, builder)?;
                Ok((v as usize, buf))
            }
            U32_CAPACITY => {
                let (v, buf) = u32::deserialize(buf, builder)?;
                Ok((v as usize, buf))
            }
            U64_CAPACITY => {
                let (v, buf) = u64::deserialize(buf, builder)?;
                Ok((v as usize, buf))
            }
            U128_CAPACITY => {
                let (v, buf) = u128::deserialize(buf, builder)?;
                Ok((v as usize, buf))
            }
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                    format!("packet length({}!={}) invalid", buf[1], capacity)))
        }
    }

}

impl Serialize for isize {
    fn raw_capacity(&self) -> usize {
        let v = *self as i128;

        if v <= i8::MAX as i128 && v >= i8::MIN as i128 {
            I8_CAPACITY
        } else if v <= i16::MAX as i128 && v >= i16::MIN as i128 {
            I16_CAPACITY
        } else if v <= i32::MAX as i128 && v >= i32::MIN as i128 {
            I32_CAPACITY
        } else if v <= i64::MAX as i128 && v >= i64::MIN as i128 {
            I64_CAPACITY
        } else {
            I128_CAPACITY
        }
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        match self.raw_capacity() {
            I8_CAPACITY => (*self as i8).serialize(buf, builder),
            I16_CAPACITY => (*self as i16).serialize(buf, builder),
            I32_CAPACITY => (*self as i32).serialize(buf, builder),
            I64_CAPACITY => (*self as i64).serialize(buf, builder),
            _ => (*self as i128).serialize(buf, builder),
        }
    }
}

impl Deserialize for isize {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        if buf.len() < SERIALIZE_HEADER_SIZE {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
        }

        let capacity = buf[1] as usize;

        match capacity {
            1 => { let (v, buf) = i8::deserialize(buf, builder)?; Ok((v as isize, buf)) }
            2 => { let (v, buf) = i16::deserialize(buf, builder)?; Ok((v as isize, buf)) }
            4 => { let (v, buf) = i32::deserialize(buf, builder)?; Ok((v as isize, buf)) }
            8 => { let (v, buf) = i64::deserialize(buf, builder)?; Ok((v as isize, buf)) }
            16 => { let (v, buf) = i128::deserialize(buf, builder)?; Ok((v as isize, buf)) }
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                    format!("packet length({}!={}) invalid", buf[1], capacity)))
        }
    }

}

impl Serialize for u8 {
    fn raw_capacity(&self) -> usize {
        U8_CAPACITY
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.to_be_bytes()); &mut cur[capacity..] };

        Ok(cur)
    }
}

impl Deserialize for u8 {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, _) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                          format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            if U8_CAPACITY as u8 != buf[1] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                          format!("packet length({}!={}) invalid", buf[1], U8_CAPACITY)));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], U8_CAPACITY)
        };

        let v = u8::from_be_bytes({
            let mut v = [0u8; 1];
            v.copy_from_slice(&end[..U8_CAPACITY]);
            v
        });


        Ok((v, &end[U8_CAPACITY..]))
    }

}

impl Serialize for u16 {
    fn raw_capacity(&self) -> usize {
        U16_CAPACITY
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.to_be_bytes()); &mut cur[capacity..] };

        Ok(cur)
    }
}

impl Deserialize for u16 {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, _) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                          format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            if U16_CAPACITY as u8 != buf[1] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                          format!("packet length({}!={}) invalid", buf[1], U16_CAPACITY)));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], U16_CAPACITY)
        };

        let v = u16::from_be_bytes({
            let mut v = [0u8; 2];
            v.copy_from_slice(&end[..U16_CAPACITY]);
            v
        });

        Ok((v, &end[U16_CAPACITY..]))
    }

}

impl Serialize for u32 {
    fn raw_capacity(&self) -> usize {
        U32_CAPACITY
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.to_be_bytes()); &mut cur[capacity..] };

        Ok(cur)
    }
}

impl Deserialize for u32 {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, _) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            if U32_CAPACITY as u8 != buf[1] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet length({}!={}) invalid", buf[1], U32_CAPACITY)));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], U32_CAPACITY)
        };

        let v = u32::from_be_bytes({
            let mut v = [0u8; 4];
            v.copy_from_slice(&end[..U32_CAPACITY]);
            v
        });

        Ok((v, &end[U32_CAPACITY..]))
    }

}

impl Serialize for u64 {
    fn raw_capacity(&self) -> usize {
        U64_CAPACITY
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.to_be_bytes()); &mut cur[capacity..] };

        Ok(cur)
    }
}

impl Deserialize for u64 {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, _) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            if U64_CAPACITY as u8 != buf[1] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet length({}!={}) invalid", buf[1], U64_CAPACITY)));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], U64_CAPACITY)
        };

        let v = u64::from_be_bytes({
            let mut v = [0u8; 8];
            v.copy_from_slice(&end[..U64_CAPACITY]);
            v
        });

        Ok((v, &end[U64_CAPACITY..]))
    }

}

impl Serialize for u128 {
    fn raw_capacity(&self) -> usize {
        U128_CAPACITY
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.to_be_bytes()); &mut cur[capacity..] };

        Ok(cur)
    }
}

impl Deserialize for u128 {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, _) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            if U128_CAPACITY as u8 != buf[1] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet length({}!={}) invalid", buf[1], U128_CAPACITY)));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], U128_CAPACITY)
        };

        let v = u128::from_be_bytes({
            let mut v = [0u8; 16];
            v.copy_from_slice(&end[..U128_CAPACITY]);
            v
        });

        Ok((v, &end[U128_CAPACITY..]))
    }

}

impl Serialize for i8 {
    fn raw_capacity(&self) -> usize {
        I8_CAPACITY
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.to_be_bytes()); &mut cur[capacity..] };

        Ok(cur)
    }
}

impl Deserialize for i8 {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, _) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            if I8_CAPACITY as u8 != buf[1] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet length({}!={}) invalid", buf[1], I8_CAPACITY)));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], I8_CAPACITY)
        };

        let v = i8::from_be_bytes({
            let mut v = [0u8; 1];
            v.copy_from_slice(&end[..I8_CAPACITY]);
            v
        });

        Ok((v, &end[I8_CAPACITY..]))
    }

}

impl Serialize for i16 {
    fn raw_capacity(&self) -> usize {
        I16_CAPACITY
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.to_be_bytes()); &mut cur[capacity..] };

        Ok(cur)
    }
}

impl Deserialize for i16 {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, _) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            if I16_CAPACITY as u8 != buf[1] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet length({}!={}) invalid", buf[1], I16_CAPACITY)));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], I16_CAPACITY)
        };

        let v = i16::from_be_bytes({
            let mut v = [0u8; 2];
            v.copy_from_slice(&end[..I16_CAPACITY]);
            v
        });

        Ok((v, &end[I16_CAPACITY..]))
    }

}

impl Serialize for i32 {
    fn raw_capacity(&self) -> usize {
        I32_CAPACITY
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.to_be_bytes()); &mut cur[capacity..] };

        Ok(cur)
    }
}

impl Deserialize for i32 {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, _) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            if I32_CAPACITY as u8 != buf[1] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet length({}!={}) invalid", buf[1], I32_CAPACITY)));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], I32_CAPACITY)
        };

        let v = i32::from_be_bytes({
            let mut v = [0u8; 4];
            v.copy_from_slice(&end[..I32_CAPACITY]);
            v
        });

        Ok((v, &end[I32_CAPACITY..]))
    }

}

impl Serialize for i64 {
    fn raw_capacity(&self) -> usize {
        I64_CAPACITY
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.to_be_bytes()); &mut cur[capacity..] };

        Ok(cur)
    }
}

impl Deserialize for i64 {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, _) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            if I64_CAPACITY as u8 != buf[1] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet length({}!={}) invalid", buf[1], I64_CAPACITY)));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], I64_CAPACITY)
        };

        let v = i64::from_be_bytes({
            let mut v = [0u8; 8];
            v.copy_from_slice(&end[..I64_CAPACITY]);
            v
        });

        Ok((v, &end[I64_CAPACITY..]))
    }

}

impl Serialize for i128 {
    fn raw_capacity(&self) -> usize {
        I128_CAPACITY
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.to_be_bytes()); &mut cur[capacity..] };

        Ok(cur)
    }
}

impl Deserialize for i128 {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, _) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            if I128_CAPACITY as u8 != buf[1] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet length({}!={}) invalid", buf[1], I128_CAPACITY)));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], I128_CAPACITY)
        };

        let v = i128::from_be_bytes({
            let mut v = [0u8; 16];
            v.copy_from_slice(&end[..I128_CAPACITY]);
            v
        });

        Ok((v, &end[I128_CAPACITY..]))
    }

}

impl Serialize for bool {
    fn raw_capacity(&self) -> usize {
        BOOL_CAPACITY
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, _) = self.serialize_head(buf, builder)?;

        let cur = { if *self { cur[0] = 1; } else { cur[0] = 0; } &mut cur[1..] };

        Ok(cur)
    }

}

impl Deserialize for bool {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, _) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            if U8_CAPACITY as u8 != buf[1] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet length({}!={}) invalid", buf[1], U8_CAPACITY)));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], U8_CAPACITY)
        };

        if end[0] == 1u8 {
            Ok((true, &end[BOOL_CAPACITY..]))
        } else {
            Ok((false, &end[BOOL_CAPACITY..]))
        }
    }

}

impl Serialize for f32 {
    fn raw_capacity(&self) -> usize {
        F32_CAPACITY
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.to_be_bytes()); &mut cur[capacity..] };

        Ok(cur)
                
    }

}

impl Deserialize for f32 {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, _) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            if F32_CAPACITY as u8 != buf[1] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet length({}!={}) invalid", buf[1], F32_CAPACITY)));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], F32_CAPACITY)
        };

        let v = f32::from_be_bytes({
            let mut v = [0u8; F32_CAPACITY];
            v.copy_from_slice(&end[..F32_CAPACITY]);
            v
        });

        Ok((v, &end[F32_CAPACITY..]))
    }

}

impl Serialize for f64 {
    fn raw_capacity(&self) -> usize {
        F64_CAPACITY
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.to_be_bytes()); &mut cur[capacity..] };

        Ok(cur)
                
    }

}

impl Deserialize for f64 {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, _) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            if F64_CAPACITY as u8 != buf[1] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet length({}!={}) invalid", buf[1], F64_CAPACITY)));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], F64_CAPACITY)
        };

        let v = f64::from_be_bytes({
            let mut v = [0u8; F64_CAPACITY];
            v.copy_from_slice(&end[..F64_CAPACITY]);
            v
        });

        Ok((v, &end[F64_CAPACITY..]))
    }

}

impl Serialize for String {
    fn raw_capacity(&self) -> usize {
        self.len()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.as_bytes()); &mut cur[capacity..] };


        Ok(cur)
    }

}

impl Deserialize for String {
    fn deserialize<'de>(buf: &'de [u8], 
                   builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, capacity) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], buf[1] as usize)
        };

        if end.len() < capacity {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
        }

        let v = {
            let mut r = vec![0u8; capacity];
            r.copy_from_slice(&end[..capacity]);
            if let Ok(v) = String::from_utf8(r) { v } else { String::new() }
        };

        Ok((v, &end[capacity..]))
    }

}

impl Serialize for &str {
    fn raw_capacity(&self) -> usize {
        self.len()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.as_bytes()); &mut cur[capacity..] };

        Ok(cur)
    }

}

impl<T> Serialize for [T]
where T: Serialize {
    fn raw_capacity(&self) -> usize {
        self.len()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, _) = self.serialize_head(buf, builder)?;

        let cur = {
            let mut length = 0usize;
            let mut child_builder = BuilderCounter::new();

            for e in self {
                length = cur.len() - e.serialize(&mut cur[length..], &mut child_builder)?.len();
            }
            &mut cur[length..]
        };

        Ok(cur)
    }

}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, capacity) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }


            (&buf[SERIALIZE_HEADER_SIZE..], buf[1] as usize)
        };

        let mut vec = Vec::with_capacity(capacity);
        let mut length = 0usize;
        let mut child_builder = BuilderCounter::new();

        for _ in 0..capacity {
            let (v, next) = T::deserialize(&end[length..], &mut child_builder)?;
            vec.push(v);
            length = end.len() - next.len();
        }

        Ok((vec, &end[length..]))
    }

}

impl<T: Serialize> Serialize for LinkedList<T> {
    fn raw_capacity(&self) -> usize {
        self.len()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, _) = self.serialize_head(buf, builder)?;

        let cur = {
            let mut length = 0usize;
            let mut child_builder = BuilderCounter::new();

            for e in self {
                length = cur.len() - e.serialize(&mut cur[length..], &mut child_builder)?.len();
            }
            &mut cur[length..]
        };

        Ok(cur)
    }

}

impl<T: Deserialize> Deserialize for LinkedList<T> {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, capacity) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                        format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            (&buf[2..], buf[1] as usize)
        };

        let mut list = LinkedList::new();
        let mut length = 0usize;
        for _ in 0..capacity {
            let (v, next) = T::deserialize(&end[length..], builder)?;
            length = end.len() - next.len();

            list.push_back(v);
        }

        Ok((list, &end[length..]))
    }

}

impl<K: Serialize, V: Serialize> Serialize for BTreeMap<K, V> {
    fn raw_capacity(&self) -> usize {
        self.len()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, _) = self.serialize_head(buf, builder)?;

        let cur = {
            let mut length = 0usize;
            let mut key_child_builder = BuilderCounter::new();
            let mut val_child_builder = BuilderCounter::new();

            for (k, v) in self {
                length = cur.len() - k.serialize(&mut cur[length..], &mut key_child_builder)?.len();
                length = cur.len() - v.serialize(&mut cur[length..], &mut val_child_builder)?.len();
            }
            &mut cur[length..]
        };

        Ok(cur)
    }

}

impl<K: Ord + Deserialize, V: Deserialize> Deserialize for BTreeMap<K, V> {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, capacity) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                        format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            (&buf[2..], buf[1] as usize)
        };

        let mut map = BTreeMap::new();
        let mut length = 0usize;
        let mut key_child_builder = BuilderCounter::new();
        let mut val_child_builder = BuilderCounter::new();
        let remain_len = end.len();

        for _ in 0..capacity {
            let (k, next) = K::deserialize(&end[length..], &mut key_child_builder)?;
            length = remain_len - next.len();
            let (v, next) = V::deserialize(&end[length..], &mut val_child_builder)?;
            length = remain_len - next.len();
            map.insert(k, v);
        }

        Ok((map, &end[length..]))
    }

}

impl<K: Serialize, V: Serialize> Serialize for HashMap<K, V> {
    fn raw_capacity(&self) -> usize {
        self.len()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, _) = self.serialize_head(buf, builder)?;

        let cur = {
            let mut length = 0usize;
            let mut key_child_builder = BuilderCounter::new();
            let mut val_child_builder = BuilderCounter::new();
    
            for (k, v) in self {
                length = cur.len() - k.serialize(&mut cur[length..], &mut key_child_builder)?.len();
                length = cur.len() - v.serialize(&mut cur[length..], &mut val_child_builder)?.len();
            }
            &mut cur[length..]
        };

        Ok(cur)
    }

}

impl<K: Eq + std::hash::Hash + Deserialize, V: Deserialize> Deserialize for HashMap<K, V> {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, capacity) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                        format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            (&buf[2..], buf[1] as usize)
        };

        let mut map = HashMap::new();
        let mut length = 0usize;
        let mut key_child_builder = BuilderCounter::new();
        let mut val_child_builder = BuilderCounter::new();
        let remain_len = end.len();
        for _ in 0..capacity {
            let (k, next) = K::deserialize(&end[length..], &mut key_child_builder)?;
            length = remain_len - next.len();
            let (v, next) = V::deserialize(&end[length..], &mut val_child_builder)?;
            length = remain_len - next.len();
            map.insert(k, v);
        }

        Ok((map, &end[length..]))
    }

}

impl<N: ArrayLength<u8>> Serialize for GenericArray<u8, N> {
    fn raw_capacity(&self) -> usize {
        self.len()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.as_slice()); &mut cur[capacity..] };

        Ok(cur)
    }
}

impl<N: ArrayLength<u8>> Deserialize for GenericArray<u8, N> {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, capacity) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], buf[1] as usize)
        };

        if end.len() < capacity {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
        }

        let v = {
            let mut r = vec![0u8; capacity];
            r.copy_from_slice(&end[..capacity]);
            GenericArray::<u8, N>::clone_from_slice(r.as_slice())
        };

        Ok((v, &end[capacity..]))
    }

}

impl<T: Serialize> Serialize for Option<T> {
    fn raw_capacity(&self) -> usize {
        0 // auto size
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        if let Some(t) = &self {
            let buf = true.serialize(buf, builder)?;

            let buf = t.serialize(buf, &mut BuilderCounter::new())?;

            Ok(buf)
        } else {
            false.serialize(buf, builder)
        }
    }

}

impl<T: Deserialize> Deserialize for Option<T> {
    fn deserialize<'de>(buf: &'de [u8], 
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (exist, buf) = bool::deserialize(buf, builder)?;

        if exist {
            let (v, buf) = T::deserialize(buf, &mut BuilderCounter::new())?;

            Ok((Some(v), buf))
        } else {
            Ok((None, buf))
        }
    }
}

#[cfg(test)]
mod test_serialize{
    use std::collections::{BTreeMap, LinkedList};
    use std::u8;

    use generic_array::{GenericArray, typenum::U32};

    use crate::codec::builder::{Serialize, Deserialize};
    use crate::codec::builder::BuilderCounter;

    #[test]
    fn t1() {
        {
            println!("GenericArray==================================================");
            let mut b = [0u8; 1024];
            let mut builder = BuilderCounter::new();

            let v1 = GenericArray::<u8, U32>::clone_from_slice("12345678901234567890123456789021".as_bytes());
            let _build_ptr = v1.serialize(&mut b, &mut builder);
            println!("wb={:?}", v1);

            let mut wb = BuilderCounter::new();
            let (wv, _end_ptr) = GenericArray::<u8, U32>::deserialize(&b, &mut wb).unwrap();
            println!("wv={:?}", wv);
        }

        {
            #[derive(Debug)]
            // let v: Vec<u8> = Vec::with_capacity(1024);
            struct GR {
                t1: u8,
                t2: u16,
                t3: bool,
                t4: u32,
                t5: u64,
                t6: i8,
                t7: i16,
                t8: i32,
                t9: i64,
                t11: u8,
                t22: u16,
                t44: u32,
                t55: u64,
                f1: f32,
                t99: isize,
                s1: String,
            }

            impl std::default::Default for GR {
                fn default() -> Self {
                    Self {
                        t1: 0u8,
                        t2: 0u16,
                        t3: false,
                        t4: 0u32,
                        t5: 0u64,
                        t6: 0i8,
                        t7: 0i16,
                        t8: 0i32,
                        t9: 0i64,
                        t11: 0u8,
                        t22: 0u16,
                        t44: 0u32,
                        t55: 0u64,
                        f1: 0.0f32,
                        t99: 0isize,
                        s1: String::new(),
                    }
        
                }
            }

            let mut buf = [0u8; 1024];
            let end = &mut buf;
            let len = end.len();
            let mut b = BuilderCounter::new();

            {
                let gr = GR {t1: u8::MAX, t2: u16::MAX, t3: true, t4: 100, t5: 98477583,
                                t6: -123i8,
                                t7: 343i16,
                                t8: -43342123i32,
                                t9: -1243877173i64,
                                t11: u8::MAX, t22: u16::MAX, t44: u32::MAX, t55: u64::MAX,
                                f1: -65525.512321,
                                t99: 65547,
                                s1: "afdfwqreqravdfqerabadsf".to_string(),
                            };
                let end = gr.t1.serialize(end, &mut b).unwrap();
                let end = gr.t2.serialize(end, &mut b).unwrap();
                let end = gr.t3.serialize(end, &mut b).unwrap();
                let end = gr.t4.serialize(end, &mut b).unwrap();
                let end = gr.t5.serialize(end, &mut b).unwrap();
                let end = gr.t6.serialize(end, &mut b).unwrap();
                let end = gr.t7.serialize(end, &mut b).unwrap();
                let end = gr.t8.serialize(end, &mut b).unwrap();
                let end = gr.t9.serialize(end, &mut b).unwrap();
                let end = gr.t11.serialize(end, &mut b).unwrap();
                let end = gr.t22.serialize(end, &mut b).unwrap();
                let end = gr.t44.serialize(end, &mut b).unwrap();
                let end = gr.t55.serialize(end, &mut b).unwrap();
                let end = gr.f1.serialize(end, &mut b).unwrap();
                let end = gr.t99.serialize(end, &mut b).unwrap();
                let end = gr.s1.serialize(end, &mut b).unwrap();

                let len = {
                    len - end.len()
                };

                println!("size={}, text={:?}", len, &buf[..len]);
            }

            let pak = &buf;

            let mut gr_c = GR::default();
            let mut gr_builder = BuilderCounter::new();

            let (t, pak) = usize::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.t1 = t as u8;

            let (t, pak) = u16::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.t2 = t;

            let (t, pak) = bool::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.t3 = t;

            let (t, pak) = u32::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.t4 = t;

            let (t, pak) = u64::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.t5 = t;

            let (t, pak) = i8::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.t6 = t;
            let (t, pak) = i16::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.t7 = t;
            let (t, pak) = i32::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.t8 = t;
            let (t, pak) = i64::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.t9 = t;

            let (t, pak) = u8::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.t11 = t;
            let (t, pak) = u16::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.t22 = t;
            let (t, pak) = u32::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.t44 = t;
            let (t, pak) = u64::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.t55 = t;

            let (t, pak) = f32::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.f1 = t;


            let (t, pak) = isize::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.t99 = t;

            let (t, pak) = String::deserialize(pak, &mut gr_builder).unwrap();
            gr_c.s1 = t;

            let _ = pak;

            println!("gr_c={:#?}", gr_c);
            // let mut gr_c = GR::default();
            // let mut gr_builder = BuilderCounter::new();
            // let (gr_c.t1, pak) = 
            // let end = gr.t2.deserialize(end, &mut b).unwrap();
            // let end = gr.t3.deserialize(end, &mut b).unwrap();
            // let end = gr.t4.deserialize(end, &mut b).unwrap();
            // let end = gr.t5.deserialize(end, &mut b).unwrap();
            // let end = gr.t6.deserialize(end, &mut b).unwrap();
            // let end = gr.t7.deserialize(end, &mut b).unwrap();
            // let end = gr.t8.deserialize(end, &mut b).unwrap();
            // let end = gr.t9.deserialize(end, &mut b).unwrap();
            // let end = gr.t11.deserialize(end, &mut b).unwrap();
            // let end = gr.t22.deserialize(end, &mut b).unwrap();
            // let end = gr.t44.deserialize(end, &mut b).unwrap();
            // let end = gr.t55.deserialize(end, &mut b).unwrap();

            // t1: u8,
            // t2: u16,
            // t3: bool,
            // t4: u32,
            // t5: u64,
            // t6: i8,
            // t7: i16,
            // t8: i32,
            // t9: i64,
            // t11: u8,
            // t22: u16,
            // t44: u32,
            // t55: u64,

        }

        {
            let mut b = [0u8; 1024];
            let mut builder = BuilderCounter::new();

            let v1:[u32; 5] = [1,3,5,7,9];
            let _ = v1.serialize(&mut b, &mut builder);

            let mut wb = BuilderCounter::new();
            let (wv, _) = Vec::<u32>::deserialize(&b, &mut wb).unwrap();
            println!("wv={:?}", wv);
        }

        {
            let mut b = [0u8; 1024];
            let mut builder = BuilderCounter::new();

            let v1 = vec!["abc", "defdfad", "efdxxasf", "er234123"];
            let _build_ptr = v1.serialize(&mut b, &mut builder);

            let mut wb = BuilderCounter::new();
            let (wv, _end_ptr) = Vec::<String>::deserialize(&b, &mut wb).unwrap();
            println!("wv={:?}", wv);
        }

        {
            println!("BTreeMap==================================================");
            let mut b = [0u8; 1024];
            let mut builder = BuilderCounter::new();

            let mut v1: BTreeMap<String, u16> = std::collections::BTreeMap::new();
            v1.insert("aaaa".to_string(), 1000u16);
            v1.insert("abbb".to_string(), 1001u16);
            v1.insert("abcc".to_string(), 1010u16);
            v1.insert("abcd".to_string(), 1100u16);
            v1.insert("bbcd".to_string(), 1101u16);
            let _build_ptr = v1.serialize(&mut b, &mut builder);
            println!("wb={:?}", b);

            let mut wb = BuilderCounter::new();
            let (wv, _end_ptr) = BTreeMap::<String, u16>::deserialize(&b, &mut wb).unwrap();
            println!("wv={:?}", wv);
        }

        {
            println!("LinkedList==================================================");
            let mut b = [0u8; 1024];
            let mut builder = BuilderCounter::new();

            let mut v1 = std::collections::LinkedList::new();
            v1.push_back("aaaa");
            v1.push_back("abbb");
            v1.push_back("abcc");
            v1.push_back("abcd");
            v1.push_back("bbcd");
            let _build_ptr = v1.serialize(&mut b, &mut builder);
            println!("wb={:?}", b);

            let mut wb = BuilderCounter::new();
            let (wv, _end_ptr) = LinkedList::<String>::deserialize(&b, &mut wb).unwrap();
            println!("wv={:?}", wv);
        }

        {
            println!("HashMap==================================================");
            let mut b = [0u8; 1024];
            let mut builder = BuilderCounter::new();

            let mut v1 = std::collections::HashMap::new();
            v1.insert("aaaa".to_string(), 1000u16);
            v1.insert("abbb".to_string(), 1001u16);
            v1.insert("abcc".to_string(), 1010u16);
            v1.insert("abcd".to_string(), 1100u16);
            v1.insert("bbcd".to_string(), 1101u16);
            let _build_ptr = v1.serialize(&mut b, &mut builder);
            println!("wb={:?}", b);

            let mut wb = BuilderCounter::new();
            let (wv, _end_ptr) = std::collections::HashMap::<String, usize>::deserialize(&b, &mut wb).unwrap();
            println!("wv={:?}", wv);
        }

    }
}
