
use aes_key::KeyMixHash;
use near_base::{device::DeviceId, *};

use crate::package::PackageBodyTrait;

#[derive(Clone, Copy)]
pub enum StunType {
    // 0x0001: 捆绑请求
    PingRequest     = 0x0001,
    // 0x0101: 捆绑响应
    PingResponse    = 0x0101,
    // 0x0111: 捆绑错误响应
    PingErrorResponse   = 0x0111,
    // 0x0002: 共享私密请求
    CallRequest     = 0x0002,
    // 0x0102: 共享私密响应
    CallResponse    = 0x0102,
    // 0x0112: 共享私密错误响应
    CallErrorResponse   = 0x0112,
    // 0x0003: 申請TurnChannel
    AllocationChannelRequest    = 0x003,
    AllocationChannelResponse   = 0x0103,
    AllocationChannelErrorResponse  = 0x0113,
}

impl Serialize for StunType {
    fn raw_capacity(&self) -> usize {
        0u16.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let v = 
            match self {
                Self::PingRequest => Self::PingRequest as u16,
                Self::PingResponse => Self::PingResponse as u16,
                Self::PingErrorResponse => Self::PingErrorResponse as u16,
                Self::CallRequest => Self::CallRequest as u16,
                Self::CallResponse => Self::CallResponse as u16,
                Self::CallErrorResponse => Self::CallErrorResponse as u16,
                Self::AllocationChannelRequest => Self::AllocationChannelRequest as u16,
                Self::AllocationChannelResponse => Self::AllocationChannelResponse as u16,
                Self::AllocationChannelErrorResponse => Self::AllocationChannelErrorResponse as u16,
            };

        v.serialize(buf)
    }
}

impl Deserialize for StunType {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = u16::deserialize(buf)?;

        let v = 
            if v == Self::PingRequest as u16 {
                Self::PingRequest
            } else if v == Self::PingResponse as u16 {
                Self::PingResponse
            } else if v == Self::PingErrorResponse as u16 {
                Self::PingErrorResponse
            } else if v == Self::CallRequest as u16 {
                Self::CallRequest
            } else if v == Self::CallResponse as u16 {
                Self::CallResponse
            } else if v == Self::CallErrorResponse as u16 {
                Self::CallErrorResponse
            } else if v == Self::AllocationChannelRequest as u16 {
                Self::AllocationChannelRequest
            } else if v == Self::AllocationChannelResponse as u16 {
                Self::AllocationChannelResponse
            } else if v == Self::AllocationChannelErrorResponse as u16 {
                Self::AllocationChannelErrorResponse
            } else {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("unmatch {v}")));
            };

        Ok((v, buf))
    }
}

impl std::fmt::Display for StunType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PingRequest => write!(f, "ping"),
            Self::PingResponse => write!(f, "ping-resp"),
            Self::PingErrorResponse => write!(f, "ping-error-resp"),
            Self::CallRequest => write!(f, "call"),
            Self::CallResponse => write!(f, "call-resp"),
            Self::CallErrorResponse => write!(f, "call-error-resp"),
            Self::AllocationChannelRequest => write!(f, "allocation-turn-channel"),
            Self::AllocationChannelResponse => write!(f, "allocation-turn-channel-resp"),
            Self::AllocationChannelErrorResponse => write!(f, "allocation-turn-channel-error-resp"),
        }
    }
}

pub const MAGIC_COOKIE: u32 = 0x2024060b;

#[derive(Clone)]
pub struct StunHead {
    stun_type: StunType,
    magic_cookie: u32,
}

impl std::default::Default for StunHead {
    fn default() -> Self {
        Self {
            stun_type: StunType::PingRequest,
            magic_cookie: MAGIC_COOKIE
        }
    }
}

#[derive(Clone, Default)]
pub struct StunBody {
    mapped_address: Option<Endpoint>,
    response_address: Option<Endpoint>,
    proxy_address: Option<Endpoint>,
    change_request: Option<bool>,
    error_code: Option<NearError>,
    mix_hash: Option<KeyMixHash>,
    live_minutes: Option<u8>,
    // username: Option<String>,
    // password: Option<String>,
    target: Option<DeviceId>,
    fromer: Option<DeviceObject>,
}

#[derive(Clone, Default)]
pub struct StunReq {
    head: StunHead,
    body: StunBody,
}

impl StunReq {

    pub fn stun_name(&self) -> String {
        format!("{}", self.head.stun_type)
    }

    pub fn new(stun_type: StunType) -> Self {
        Self {
            head: StunHead { stun_type: stun_type, ..Default::default() },
            body: Default::default(),
        }
    }

    pub fn set_mapped_address(mut self, mapped_address: Option<Endpoint>) -> Self {
        self.body.mapped_address = mapped_address;
        self
    }

    pub fn set_response_address(mut self, response_address: Option<Endpoint>) -> Self {
        self.body.response_address = response_address;
        self
    }

    pub fn set_proxy_address(mut self, proxy_address: Option<Endpoint>) -> Self {
        self.body.proxy_address = proxy_address;
        self
    }

    pub fn set_change_request(mut self, change_request: Option<bool>) -> Self {
        self.body.change_request = change_request;
        self
    }

    pub fn set_error_code(mut self, error_code: Option<NearError>) -> Self {
        self.body.error_code = error_code;
        self
    }

    pub fn set_mixhash(mut self, mix_hash: Option<KeyMixHash>) -> Self {
        self.body.mix_hash = mix_hash;
        self
    }

    pub fn set_live_minutes(mut self, live_minutes: Option<u8>) -> Self {
        self.body.live_minutes = live_minutes;
        self
    }


    // pub fn set_username(mut self, username: Option<String>) -> Self {
    //     self.body.username = username;
    //     self
    // }

    // pub fn set_password(mut self, password: Option<String>) -> Self {
    //     self.body.password = password;
    //     self
    // }

    pub fn set_target(mut self, target: Option<DeviceId>) -> Self {
        self.body.target = target;
        self
    }

    pub fn set_fromer(mut self, fromer: Option<DeviceObject>) -> Self {
        self.body.fromer = fromer;
        self
    }

    pub fn stun_type(&self) -> StunType {
        self.head.stun_type
    }

    pub fn take_mapped_address(&mut self) -> Option<Endpoint> {
        std::mem::replace(&mut self.body.mapped_address, None)
    }

    pub fn take_response_address(&mut self) -> Option<Endpoint> {
        std::mem::replace(&mut self.body.response_address, None)
    }

    pub fn take_proxy_address(&mut self) -> Option<Endpoint> {
        std::mem::replace(&mut self.body.proxy_address, None)
    }

    pub fn take_change_request(&mut self) -> Option<bool> {
        std::mem::replace(&mut self.body.change_request, None)
    }

    pub fn take_error_code(&mut self) -> Option<NearError> {
        std::mem::replace(&mut self.body.error_code, None)
    }

    // pub fn take_username(&mut self) -> Option<String> {
    //     std::mem::replace(&mut self.body.username, None)
    // }

    // pub fn take_password(&mut self) -> Option<String> {
    //     std::mem::replace(&mut self.body.password, None)
    // }

    pub fn take_mixhash(&mut self) -> Option<KeyMixHash> {
        std::mem::replace(&mut self.body.mix_hash, None)
    }

    pub fn take_live_minutes(&mut self) -> Option<u8> {
        self.body.live_minutes
    }

    pub fn take_target(&mut self) -> Option<DeviceId> {
        std::mem::replace(&mut self.body.target, None)
    }

    pub fn take_fromer(&mut self) -> Option<DeviceObject> {
        std::mem::replace(&mut self.body.fromer, None)
    }

}

impl Serialize for StunReq {
    fn raw_capacity(&self) -> usize {
        Self::version().raw_capacity() +
        self.head.stun_type.raw_capacity() + 
        self.head.magic_cookie.raw_capacity() + 
        self.body.mapped_address.raw_capacity() + 
        self.body.response_address.raw_capacity() + 
        self.body.proxy_address.raw_capacity() +
        self.body.change_request.raw_capacity() + 
        self.body.error_code.raw_capacity() + 
        self.body.mix_hash.raw_capacity() +
        self.body.live_minutes.raw_capacity() +
        // self.body.username.raw_capacity() + 
        // self.body.password.raw_capacity() + 
        self.body.target.raw_capacity() + 
        self.body.fromer.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = Self::version().serialize(buf)?;
        let buf = self.head.stun_type.serialize(buf)?;
        let buf = self.head.magic_cookie.serialize(buf)?;
        let buf = self.body.mapped_address.serialize(buf)?;
        let buf = self.body.response_address.serialize(buf)?;
        let buf = self.body.proxy_address.serialize(buf)?;
        let buf = self.body.change_request.serialize(buf)?;
        let buf = self.body.error_code.serialize(buf)?;
        let buf = self.body.mix_hash.serialize(buf)?;
        let buf = self.body.live_minutes.serialize(buf)?;
        // let buf = self.body.username.serialize(buf)?;
        // let buf = self.body.password.serialize(buf)?;
        let buf = self.body.target.serialize(buf)?;
        let buf = self.body.fromer.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for StunReq {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = u8::deserialize(buf)?;

        if v != Self::version() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("unmatch version: got:{}, expr:{}", Self::version(), v)));
        }

        let (stun_type, buf) = StunType::deserialize(buf)?;
        let (magic_cookie, buf) = u32::deserialize(buf)?;

        if magic_cookie != MAGIC_COOKIE {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN, format!("unknown magic cookie, exper: {magic_cookie}")));
        }

        let (mapped_address, buf) = Option::<Endpoint>::deserialize(buf)?;
        let (response_address, buf) = Option::<Endpoint>::deserialize(buf)?;
        let (proxy_address, buf) = Option::<Endpoint>::deserialize(buf)?;
        let (change_request, buf) = Option::<bool>::deserialize(buf)?;
        let (error_code, buf) = Option::<NearError>::deserialize(buf)?;
        let (mix_hash, buf) = Option::<KeyMixHash>::deserialize(buf)?;
        let (live_minutes, buf) = Option::<u8>::deserialize(buf)?;
        // let (username, buf) = Option::<String>::deserialize(buf)?;
        // let (password, buf) = Option::<String>::deserialize(buf)?;
        let (target, buf) = Option::<DeviceId>::deserialize(buf)?;
        let (fromer, buf) = Option::<DeviceObject>::deserialize(buf)?;

        Ok((Self{
            head: StunHead { stun_type, magic_cookie },
            body: StunBody { mapped_address, response_address, proxy_address, change_request, error_code, mix_hash, live_minutes, target, fromer },
        }, buf))
    
    }
}

impl PackageBodyTrait for StunReq {
    fn version() -> u8 {
        1
    }
}

impl std::fmt::Display for StunReq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "StunReq: head: [stun_type: {}], body: [mapped_address: {:?}, response_address: {:?}, proxy_address: {:?}, change_request: {:?}, error_code: {:?}, mix_hash: {:?}, live_minutes: {:?}, target: {:?}, fromer: {:?}, have_fromer: {}]",
            self.head.stun_type,
            self.body.mapped_address,
            self.body.response_address,
            self.body.proxy_address,
            self.body.change_request,
            self.body.error_code,
            self.body.mix_hash,
            self.body.live_minutes,
            self.body.target,
            self.body.fromer.as_ref().map(| fromer | fromer.object_id()),
            self.body.fromer.is_some(),
        )
    }
}
