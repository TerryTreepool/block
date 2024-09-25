
use near_base::{Serialize, Deserialize, RawFixedBytes,
                NearResult, NearError, ErrorCode,
    };
use near_transport::ItfTrait;

#[derive(Clone, Copy, Default)]
pub enum MessageExpire {
    #[default]
    Forever,
    Onetime,
    ExpireTime(u64),
    Normal,
}

impl std::fmt::Debug for MessageExpire {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Forever => write!(f, "Forever"),
            Self::Onetime => write!(f, "Onetime"),
            Self::ExpireTime(v) => write!(f, "Expire Time {}", v),
            Self::Normal => write!(f, "Normal"),
        }
    }
}

impl Serialize for MessageExpire {
    fn raw_capacity(&self) -> usize {
        match self {
            MessageExpire::Forever | MessageExpire::Onetime | MessageExpire::Normal => 0u8.raw_capacity(),
            MessageExpire::ExpireTime(v) => {
                0u8.raw_capacity() + v.raw_capacity()
            }
        }
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let (code, value) = match self {
            MessageExpire::Forever => (1u8, None),
            MessageExpire::Onetime => (2u8, None),
            MessageExpire::ExpireTime(v) => (3u8, Some(v)),
            MessageExpire::Normal => (4u8, None),
        };

        let buf = if let Some(value) = value {
            let buf = code.serialize(buf)?;
            let buf = value.serialize(buf)?;
            buf
        } else {
            let buf = code.serialize(buf)?;
            buf
        };

        Ok(buf)
    }
}

impl Deserialize for MessageExpire {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (code, buf) = u8::deserialize(buf)?;

        match code {
            1u8 => Ok((Self::Forever, buf)),
            2u8 => Ok((Self::Onetime, buf)),
            3u8 => {
                let (v, buf) = u64::deserialize(buf)?;
                Ok((Self::ExpireTime(v), buf))
            }
            4u8 => Ok((Self::Normal, buf)),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("[{}] is invalide message expire code", code)))
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum MessageType {
    #[default]
    Public = 0,
    Private = 1,
}

impl Serialize for MessageType {
    fn raw_capacity(&self) -> usize {
        i32::raw_bytes()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        (*self as i32).serialize(buf)
    }
}

impl Deserialize for MessageType {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = i32::deserialize(buf)?;

        Ok((Self::try_from(v)?, buf))
    }
}

impl TryFrom<i32> for MessageType {
    type Error = NearError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Public),
            1 => Ok(Self::Private),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, format!("undefined {value} message type")))
        }
    }
}

impl std::fmt::Debug for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "Public Message"),
            Self::Private => write!(f, "Private Message"),
        }
    }
}

#[derive(Clone)]
pub struct SubscribeMessage {
    pub message_list: Vec<(String, MessageExpire)>,
}

impl std::fmt::Display for SubscribeMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v: Vec<&String> =   self.message_list
                                    .iter()
                                    .map(| (message, _) | {
                                        message
                                    })
                                    .collect();

        write!(f, "{:?}", v)
    }
}

impl ItfTrait for SubscribeMessage {}

impl Serialize for SubscribeMessage {
    fn raw_capacity(&self) -> usize {
        self.message_list.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.message_list.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for SubscribeMessage {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (message_list, buf) = Vec::<(String, MessageExpire)>::deserialize(buf)?;

        Ok((Self{
            message_list
        }, buf))
    }

}

pub struct DissubcribeMessage {
    pub message: String,
}

impl ItfTrait for DissubcribeMessage {}

impl Serialize for DissubcribeMessage {
    fn raw_capacity(&self) -> usize {
        self.message.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.message.serialize(buf)?;
        Ok(buf)
    }
}

impl Deserialize for DissubcribeMessage {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (message, buf) = String::deserialize(buf)?;

        Ok((Self{message}, buf))
    }
}

