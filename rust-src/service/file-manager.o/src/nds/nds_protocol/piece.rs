
use std::io::{Cursor, Read, BufRead};

use futures::AsyncReadExt;
use near_base::{ChunkId, Deserialize, Serialize, NearResult, NearError, ErrorCode, RawFixedBytes, builder_codec::SerializeWithContext};
use near_transport::{ItfTrait, ItfBuilderTrait};

// sync protocol
#[derive(Clone)]
pub struct InterestMessage {
    pub chunk: ChunkId,
    pub desc: ChunkPieceDesc,
}

impl ItfTrait for InterestMessage {}

impl Serialize for InterestMessage {
    fn raw_capacity(&self) -> usize {
        self.chunk.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.chunk.serialize(buf)?;
        let buf = self.desc.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for InterestMessage {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (chunk, buf) = ChunkId::deserialize(buf)?;
        let (desc, buf) = ChunkPieceDesc::deserialize(buf)?;

        Ok((Self{
            chunk,
            desc,
        }, buf))
    }

}

pub struct InterestMessageResponse {
    pub chunk: ChunkId,
    pub errno: Option<NearError>,
}

impl ItfTrait for InterestMessageResponse {}

impl Serialize for InterestMessageResponse {
    fn raw_capacity(&self) -> usize {
        self.chunk.raw_capacity() + self.errno.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.chunk.serialize(buf)?;
        let buf = self.errno.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for InterestMessageResponse {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (chunk, buf) = ChunkId::deserialize(buf)?;
        let (errno, buf) = Option::<NearError>::deserialize(buf)?;

        Ok((Self{
            chunk,
            errno,
        }, buf))
    }

}

#[derive(Clone)]
pub enum ChunkPieceDesc {
    Range(u32 /*offset*/, u32 /*length*/),
}

impl RawFixedBytes for ChunkPieceDesc {
    fn raw_min_bytes() -> usize {
        u8::raw_min_bytes() + u32::raw_min_bytes() + u32::raw_min_bytes()
    }
}

impl ChunkPieceDesc {
    #[allow(unused)]
    pub fn to_range(&self) -> Option<(u32, u32)> {
        match &self {
            Self::Range(index, count) => Some((*index, *count)),
        }
    }
}

impl Serialize for ChunkPieceDesc {
    fn raw_capacity(&self) -> usize {
        match &self {
            Self::Range(index, count) => {
                1u8.raw_capacity() + index.raw_capacity() + count.raw_capacity()
            }
        }
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        match &self {
            Self::Range(index, count) => {
                let buf = 1u8.serialize(buf)?;
                let buf = index.serialize(buf)?;
                let buf = count.serialize(buf)?;
                Ok(buf)
            }
        }
    }

}

impl Deserialize for ChunkPieceDesc {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (t, buf) = u8::deserialize(buf)?;

        match t {
            1u8 => {
                let (index, buf) = u32::deserialize(buf)?;
                let (count, buf) = u32::deserialize(buf)?;
                Ok((Self::Range(index, count), buf))
            }
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("Cloud not match [{}] piece desc type", t)))
            }
        }
    }
}

pub struct PieceMessage {
    pub chunk: ChunkId,
    pub desc: ChunkPieceDesc,
    pub data: Vec<u8>,
}

impl RawFixedBytes for PieceMessage {
    fn raw_min_bytes() -> usize {
        ChunkId::raw_min_bytes() + ChunkPieceDesc::raw_min_bytes() + u16::raw_min_bytes()
    }
}

impl ItfTrait for PieceMessage {}

impl Serialize for PieceMessage {
    fn raw_capacity(&self) -> usize {
        self.chunk.raw_capacity() +
        self.desc.raw_capacity() +
        self.data.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.chunk.serialize(buf)?;
        let buf = self.desc.serialize(buf)?;
        let buf = self.data.serialize(buf)?;
        Ok(buf)
    }

}

impl Deserialize for PieceMessage {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (chunk, buf) = ChunkId::deserialize(buf)?;
        let (desc, buf) = ChunkPieceDesc::deserialize(buf)?;
        let (data, buf) = Vec::<u8>::deserialize(buf)?;

        Ok((Self{
            chunk, desc, data
        }, buf))
    }
}

pub struct PieceMessageBuilder {
    pub chunk: ChunkId,
    pub offset: usize,
    pub data: Vec<u8>,
}

impl ItfBuilderTrait for PieceMessageBuilder {
    type R = PieceMessage;
    fn build(&self, size: usize) -> Vec<Self::R> {
        let r = {
            let min_bytes = PieceMessage::raw_min_bytes();
            let remain_bytes = size - min_bytes;
            let mut ret_array = vec![];

            fn split(data: &[u8], at: usize, ret: &mut Vec<Vec<u8>>) {
                if data.len() < at {
                    ret.push(data.to_vec());
                } else {
                    let (l, r) = data.split_at(at);
                    ret.push(l.to_vec());
                    split(r, at, ret);
                }
            }

            split(&self.data, remain_bytes, &mut ret_array);

            ret_array
        };

        let mut offset = self.offset;
        let mut new_offset = offset;

        let mut ret_array = vec![];
        for it in r {
            let a = PieceMessage {
                chunk: self.chunk.clone(),
                desc: ChunkPieceDesc::Range(offset as u32, {
                    offset += it.len();
                    offset as u32
                }),
                data: it,
            };
            ret_array.push(a);
        }

        ret_array
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PieceControlCommand {
    Continue,
    Finish,
    Pause,
    Cancel,
}

impl TryFrom<u8> for PieceControlCommand {
    type Error = NearError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match &value {
            1u8 => Ok(PieceControlCommand::Continue),
            2u8 => Ok(PieceControlCommand::Finish),
            3u8 => Ok(PieceControlCommand::Pause),
            4u8 => Ok(PieceControlCommand::Cancel),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("Cloud not match [{}] piece control command", value)))
        }
    }
}

impl PieceControlCommand {
    pub fn into_u8(self) -> u8 {
        match &self {
            PieceControlCommand::Continue => 1u8,
            PieceControlCommand::Finish => 2u8,
            PieceControlCommand::Pause => 3u8,
            PieceControlCommand::Cancel => 4u8,
        }
    }
}

impl Serialize for PieceControlCommand {
    fn raw_capacity(&self) -> usize {
        0u8.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        self.into_u8().serialize(buf)
    }
}

impl Deserialize for PieceControlCommand {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (ctrl, buf) = u8::deserialize(buf)?;

        Ok((PieceControlCommand::try_from(ctrl)?, buf))
    }
}

pub struct PieceMessageResponse {
    pub chunk: ChunkId,
    pub command: PieceControlCommand,
    pub lost_index: Option<Vec<u32>>
}

impl ItfTrait for PieceMessageResponse {}

impl Serialize for PieceMessageResponse {
    fn raw_capacity(&self) -> usize {
        self.chunk.raw_capacity() +
        self.command.raw_capacity() +
        self.lost_index.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.chunk.serialize(buf)?;
        let buf = self.command.serialize(buf)?;
        let buf = self.lost_index.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for PieceMessageResponse {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (chunk, buf) = ChunkId::deserialize(buf)?;
        let (command, buf) = PieceControlCommand::deserialize(buf)?;
        let (lost_index, buf) = Option::<Vec<u32>>::deserialize(buf)?;

        Ok((Self{
            chunk, command, lost_index,
        }, buf))
    }
}
