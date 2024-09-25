
use near_base::{ChunkId, Deserialize, Serialize, NearResult, NearError, ErrorCode, RawFixedBytes, };
use near_transport::{ItfTrait, ItfBuilderTrait, };

#[derive(Clone, Copy, Default)]
pub struct SessionData {
    pub session_id: u32,
    pub session_sub_id: u32,
}

impl std::fmt::Display for SessionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{id={}, sub_id={}}}", self.session_id, self.session_sub_id)
    }
}

impl RawFixedBytes for SessionData {
    fn raw_bytes() -> usize {
        u32::raw_bytes() + u32::raw_bytes()
    }
}

impl Serialize for SessionData {
    fn raw_capacity(&self) -> usize {
        self.session_id.raw_capacity() +
        self.session_sub_id.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf  = self.session_id.serialize(buf)?;
        let buf = self.session_sub_id.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for SessionData {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (session_id, buf) = u32::deserialize(buf)?;
        let (session_sub_id, buf) = u32::deserialize(buf)?;

        Ok((Self{
            session_id,
            session_sub_id,
        }, buf))
    }

}

// sync protocol
#[derive(Clone, Default)]
pub struct InterestMessage {
    pub session_data: SessionData,
    pub chunk: ChunkId,
    pub encoder: ChunkEncodeDesc,
}

impl std::fmt::Display for InterestMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "chunk: {}, session: {}, encoder: {}", self.chunk, self.session_data, self.encoder)
    }
}

impl RawFixedBytes for InterestMessage {
    fn raw_bytes() -> usize {
        SessionData::raw_bytes() +
        ChunkId::raw_bytes() +
        ChunkEncodeDesc::raw_bytes()
    }
}

impl Serialize for InterestMessage {
    fn raw_capacity(&self) -> usize {
        self.session_data.raw_capacity() +
        self.chunk.raw_capacity() + 
        self.encoder.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf  = self.session_data.serialize(buf)?;
        let buf = self.chunk.serialize(buf)?;
        let buf = self.encoder.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for InterestMessage {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (session_data, buf) = SessionData::deserialize(buf)?;
        let (chunk, buf) = ChunkId::deserialize(buf)?;
        let (encoder, buf) = ChunkEncodeDesc::deserialize(buf)?;

        Ok((Self{
            session_data,
            chunk,
            encoder,
        }, buf))
    }

}

#[test]
fn test_interest_message() {
    let m = InterestMessage {
        session_data: Default::default(),
        chunk: ChunkId::default(),
        encoder: ChunkEncodeDesc::Stream(Default::default())
    };
    let len = m.raw_capacity();
    let mut v = vec![0u8; len];
    let _ = m.serialize(v.as_mut_slice()).unwrap();
}

pub struct InterestMessageResponse {
    pub chunk: ChunkId,
}

impl Serialize for InterestMessageResponse {
    fn raw_capacity(&self) -> usize {
        self.chunk.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.chunk.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for InterestMessageResponse {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (chunk, buf) = ChunkId::deserialize(buf)?;

        Ok((Self{
            chunk,
        }, buf))
    }

}

#[derive(Clone)]
pub enum ChunkEncodeDesc {
    // Stream(u32 /* chunk offset size */, u32 /* chunk count size */)
    Stream(ChunkRange)
}

impl std::default::Default for ChunkEncodeDesc {
    fn default() -> Self {
        Self::Stream(ChunkRange::default())
    }
}

impl std::fmt::Display for ChunkEncodeDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stream(range) => {
                range.fmt(f)
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct ChunkRange {
    pub start: u32,
    pub end: u32,
}

impl ChunkRange {
    pub fn contains(&self, other: &ChunkRange) -> bool {
        if other.start >= self.start && other.start < self.end {
            if other.end > self.start && other.end <= self.end {
                return true;
            }
        } 

        return false;
    }

}

impl std::fmt::Display for ChunkRange {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)        
    }

}

impl ChunkEncodeDesc {
    pub fn create_stream(chunk: &ChunkId) -> Self {
        let chunk_len = chunk.len();
        let piece_len = PieceMessage::payload_max_len();

        let end = if chunk_len % piece_len == 0 {
            chunk_len / piece_len
        } else {
            (chunk_len / piece_len) + 1
        };

        // Self::Stream(0, index_max as u16)
        Self::Stream(ChunkRange { start: 0, end: end as u32})
    }

}

impl RawFixedBytes for ChunkEncodeDesc {
    fn raw_bytes() -> usize {
        u8::raw_bytes() + u32::raw_bytes() + u32::raw_bytes()
    }
}

impl Serialize for ChunkEncodeDesc {
    fn raw_capacity(&self) -> usize {
        match &self {
            Self::Stream(range) => {
                1u8.raw_capacity() + range.start.raw_capacity() + range.end.raw_capacity()
            }
        }
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        match &self {
            Self::Stream(range) => {
                let buf = 1u8.serialize(buf)?;
                let buf = range.start.serialize(buf)?;
                let buf = range.end.serialize(buf)?;
                
                Ok(buf)
            }
        }
    }

}

impl Deserialize for ChunkEncodeDesc {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (t, buf) = u8::deserialize(buf)?;

        match t {
            1u8 => {
                let (start, buf) = u32::deserialize(buf)?;
                let (end, buf) = u32::deserialize(buf)?;
                Ok((Self::Stream(ChunkRange{start, end}), buf))
            }
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("Cloud not match [{}] piece desc type", t)))
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum PieceEncodeDesc {
    Range(u16 /*piece index*/, u16 /*piece size*/),
}

impl std::default::Default for PieceEncodeDesc {
    fn default() -> Self {
        Self::Range(0, 0)
    }
}

impl RawFixedBytes for PieceEncodeDesc {
    fn raw_bytes() -> usize {
        u8::raw_bytes() + u16::raw_bytes() + u16::raw_bytes()
    }
}

impl PieceEncodeDesc {
    #[allow(unused)]
    pub fn to_range(&self) -> Option<(u16, u16)> {
        match &self {
            Self::Range(index, count) => Some((*index, *count)),
        }
    }
}

impl Serialize for PieceEncodeDesc {
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

impl Deserialize for PieceEncodeDesc {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (t, buf) = u8::deserialize(buf)?;

        match t {
            1u8 => {
                let (index, buf) = u16::deserialize(buf)?;
                let (count, buf) = u16::deserialize(buf)?;
                Ok((Self::Range(index, count), buf))
            }
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("Cloud not match [{}] piece desc type", t)))
            }
        }
    }
}

impl std::fmt::Display for PieceEncodeDesc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Range(index, length) => {
                write!(f, "Range:{{index={}, length={}}}", index, length)
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct PieceMessage {
    pub session_data: SessionData,
    pub chunk: ChunkId,
    pub desc: PieceEncodeDesc,
    pub data: Vec<u8>,
}

impl std::fmt::Display for PieceMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "chunk:{}, session: {}, desc: {},data-len: {}", 
                self.chunk, 
                self.session_data, 
                self.desc, 
                self.data.len())
    }
}

impl PieceMessage {
    pub fn payload_max_len() -> usize {
        2000
    }

}

impl Serialize for PieceMessage {
    fn raw_capacity(&self) -> usize {
        self.session_data.raw_capacity() +
        self.chunk.raw_capacity() +
        self.desc.raw_capacity() +
        self.data.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.session_data.serialize(buf)?;
        let buf = self.chunk.serialize(buf)?;
        let buf = self.desc.serialize(buf)?;
        let buf = self.data.serialize(buf)?;
        Ok(buf)
    }

}

impl Deserialize for PieceMessage {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (session_data, buf) = SessionData::deserialize(buf)?;
        let (chunk, buf) = ChunkId::deserialize(buf)?;
        let (desc, buf) = PieceEncodeDesc::deserialize(buf)?;
        let (data, buf) = Vec::<u8>::deserialize(buf)?;

        Ok((Self{
            session_data, chunk, desc, data
        }, buf))
    }
}

pub struct PieceMessageBuilder<'a> {
    pub session_data: SessionData,
    pub chunk: &'a ChunkId,
    pub encoder: ChunkEncodeDesc,
    pub data: Vec<u8>,
}

impl<'a> ItfBuilderTrait for PieceMessageBuilder<'a> {

    type R = PieceMessage;

    fn build(&self) -> Vec<Self::R> {
        let r = {
            fn split<'a>(data: &'a [u8], at: usize) -> Vec<&'a [u8]> {
                let mut array = vec![];
                let mut end = data;

                loop {
                    let data_len = end.len();
                    if data_len < at {
                        array.push(end);
                        break;
                    } else {
                        let (l, r) = end.split_at(at);
                        array.push(l);
                        end = r;
                    }
                }

                array
            }

            split(&self.data, Self::R::payload_max_len())
        };

        let mut ret_array = vec![];
        let mut length = 0;
        for (index, &it) in r.iter().enumerate() {
            let desc = 
            match &self.encoder {
                ChunkEncodeDesc::Stream(range) => {
                    let r = PieceEncodeDesc::Range(index as u16, it.len() as u16);
                    length += it.len();
                    r
                }
            };

            ret_array.push(PieceMessage {
                session_data: self.session_data,
                chunk: self.chunk.clone(),
                desc,
                data: it.to_vec()
            })   
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
