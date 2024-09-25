
use near_base::{*, device::DeviceId};

use crate::package::PackageBodyTrait;

#[derive(Clone, Default)]
pub struct Ping {
    pub session_id: u64,
    pub send_time: u64,
    pub ping_sequence: u32,
    pub peer_id: DeviceId,
    pub peer_info: Option<DeviceObject>,    //发送者设备信息
    pub nonce: String,
}

impl Serialize for Ping {
    fn raw_capacity(&self) -> usize {
        Self::version().raw_capacity() +
        self.session_id.raw_capacity() + 
        self.send_time.raw_capacity() +
        self.ping_sequence.raw_capacity() + 
        self.peer_id.raw_capacity() + 
        self.peer_info.raw_capacity() +
        self.nonce.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = Self::version().serialize(buf)?;
        let buf = self.session_id.serialize(buf)?;
        let buf = self.send_time.serialize(buf)?;
        let buf = self.ping_sequence.serialize(buf)?;
        let buf = self.peer_id.serialize(buf)?;
        let buf = self.peer_info.serialize(buf)?;
        let buf = self.nonce.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for Ping {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = u8::deserialize(buf)?;

        if v != Self::version() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("unmatch version: got:{}, expr:{}", Self::version(), v)));
        }
    
        let (session_id, buf) = u64::deserialize(buf)?;
        let (send_time, buf) = Timestamp::deserialize(buf)?;
        let (ping_sequence, buf) = u32::deserialize(buf)?;
        let (peer_id, buf) = DeviceId::deserialize(buf)?;
        let (peer_info, buf) = Option::<DeviceObject>::deserialize(buf)?;
        let (nonce, buf) = String::deserialize(buf)?;

        Ok((Self{
            session_id,
            send_time, ping_sequence, 
            peer_id,
            peer_info,
            nonce,
        }, buf))

    }

}

impl std::fmt::Display for Ping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ping: version: {}, session_id:{}, send_time: {}, ping_sequence: {}, peer_id: {:?}, nonce: {}", 
            Self::version(), 
            self.session_id,
            self.send_time, 
            self.ping_sequence, 
            self.peer_id,
            self.nonce
        )
    }
}

impl PackageBodyTrait for Ping {
    fn version() -> u8 {
        1u8
    }
}

// ping resp
#[derive(Clone, Default)]
pub struct PingResp {

    pub session_id: u64,
    pub ping_sequence: u32,
    pub peer_id: DeviceId,
    pub reverse_endpoint: Option<Endpoint>,

}

impl Serialize for PingResp {
    fn raw_capacity(&self) -> usize {
        Self::version().raw_capacity() +
        self.session_id.raw_capacity() +
        self.ping_sequence.raw_capacity() +
        self.peer_id.raw_capacity() +
        self.reverse_endpoint.raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = Self::version().serialize(buf)?;
        let buf = self.session_id.serialize(buf)?;
        let buf = self.ping_sequence.serialize(buf)?;
        let buf = self.peer_id.serialize(buf)?;
        let buf = self.reverse_endpoint.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for PingResp {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = u8::deserialize(buf)?;

        if v != Self::version() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("unmatch version: got:{}, expr:{}", Self::version(), v)));
        }
    
        let (session_id, buf) = u64::deserialize(buf)?;
        let (ping_sequence, buf) = u32::deserialize(buf)?;
        let (peer_id, buf) = DeviceId::deserialize(buf)?;
        let (reverse_endpoint, buf) = Option::<Endpoint>::deserialize(buf)?;

        Ok((Self{
            session_id,
            ping_sequence,
            peer_id,
            reverse_endpoint,
        }, buf))
    }

}

impl std::fmt::Display for PingResp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ping: version: {}, ping_sequence: {}, reverse_endpoint: {:?}", 
            Self::version(), 
            self.ping_sequence, 
            self.reverse_endpoint, 
        )
    }
}

impl PackageBodyTrait for PingResp {
    fn version() -> u8 {
        1u8
    }
}

