use near_base::{device::DeviceId, Deserialize, DeviceObject, ErrorCode, NearError, Serialize, Timestamp};

use crate::package::PackageBodyTrait;


#[derive(Clone, Default)]
pub struct CallReq {
    pub session_id: u64,
    pub call_sequence: u32,
    pub to_peer_id: DeviceId,
    pub fromer: Option<DeviceObject>,
    // pub peer_info: Option<DeviceObject>,
    pub call_time: Timestamp,
}

impl Serialize for CallReq {
    fn raw_capacity(&self) -> usize {
        Self::version().raw_capacity() +
        self.session_id.raw_capacity() +
        self.call_sequence.raw_capacity() +
        self.to_peer_id.raw_capacity() +
        self.fromer.raw_capacity() +
        self.call_time.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> near_base::NearResult<&'a mut [u8]> {
        let buf = Self::version().serialize(buf)?;
        let buf = self.session_id.serialize(buf)?;
        let buf = self.call_sequence.serialize(buf)?;
        let buf = self.to_peer_id.serialize(buf)?;
        let buf = self.fromer.serialize(buf)?;
        let buf = self.call_time.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for CallReq {
    fn deserialize<'de>(buf: &'de [u8]) -> near_base::NearResult<(Self, &'de [u8])> {
        let (v, buf) = u8::deserialize(buf)?;

        if v != Self::version() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("unmatch version: got:{}, expr:{}", Self::version(), v)));
        }

        let (session_id, buf) = u64::deserialize(buf)?;
        let (call_sequence, buf) = u32::deserialize(buf)?;
        let (to_peer_id, buf) = DeviceId::deserialize(buf)?;
        let (fromer, buf) = Option::<DeviceObject>::deserialize(buf)?;
        let (call_time, buf) = Timestamp::deserialize(buf)?;

        Ok((Self{
            session_id,
            call_sequence,
            to_peer_id,
            fromer,
            call_time,
        }, buf))
    
    }
}

impl PackageBodyTrait for CallReq {
    fn version() -> u8 {
        1
    }
}

impl std::fmt::Display for CallReq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "CallReq: session_id: {}, call_sequence: {}, to_peer_id: {}, have_fromer: {}, call_time: {}",
            self.session_id,
            self.call_sequence,
            self.to_peer_id,
            self.fromer.is_some(),
            self.call_time
        )
    }
}

#[derive(Clone, Default)]
pub struct CallResp {
//     //sn call的响应包
    pub session_id: u64,
    pub call_sequence: u32,
    pub result: u8,                   //
    pub to_peer_info: Option<DeviceObject>, //    
}

impl Serialize for CallResp {
    fn raw_capacity(&self) -> usize {
        Self::version().raw_capacity() +
        self.session_id.raw_capacity() +
        self.call_sequence.raw_capacity() +
        self.result.raw_capacity() +
        self.to_peer_info.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> near_base::NearResult<&'a mut [u8]> {
        let buf = Self::version().serialize(buf)?;
        let buf = self.session_id.serialize(buf)?;
        let buf = self.call_sequence.serialize(buf)?;
        let buf = self.result.serialize(buf)?;
        let buf = self.to_peer_info.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for CallResp {
    fn deserialize<'de>(buf: &'de [u8]) -> near_base::NearResult<(Self, &'de [u8])> {
        let (v, buf) = u8::deserialize(buf)?;

        if v != Self::version() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("unmatch version: got:{}, expr:{}", Self::version(), v)));
        }

        let (session_id, buf) = u64::deserialize(buf)?;
        let (call_sequence, buf) = u32::deserialize(buf)?;
        let (result, buf) = u8::deserialize(buf)?;
        let (to_peer_info, buf) = Option::<DeviceObject>::deserialize(buf)?;

        Ok((Self{
            session_id,
            call_sequence,
            result,
            to_peer_info,
        }, buf))
    
    }
}

impl PackageBodyTrait for CallResp {
    fn version() -> u8 {
        1
    }
}

impl std::fmt::Display for CallResp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "CallResp: session_id: {}, call_sequence: {}, result: {}, have_to_peer_info: {}, to_peer_info: {:?}",
            self.session_id,
            self.call_sequence,
            self.result,
            self.to_peer_info.is_some(),
            self.to_peer_info.as_ref().map(| peer_info | peer_info.object_id())
        )
    }
}

#[derive(Clone, Default)]
pub struct CalledReq {
    pub peer_info: DeviceObject,
    pub call_sequence: u32,
    pub call_time: Timestamp,
    // pub seq: TempSeq,
    // pub sn_peer_id: DeviceId,
    // pub to_peer_id: DeviceId,
    // pub reverse_endpoint_array: Vec<Endpoint>,
    // pub active_pn_list: Vec<DeviceId>,
    // pub peer_info: Device,
    // pub call_seq: TempSeq,
    // pub call_send_time: Timestamp,
    // pub payload: SizedOwnedData<SizeU16>,

    // pub session_id: u64,
    // pub call_sequence: u32,
    // pub to_peer_id: DeviceId,
    // pub peer_info: Option<DeviceObject>,
    // pub call_time: Timestamp,
}

impl Serialize for CalledReq {
    fn raw_capacity(&self) -> usize {
        Self::version().raw_capacity() +
        self.peer_info.raw_capacity() +
        self.call_sequence.raw_capacity() +
        self.call_time.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> near_base::NearResult<&'a mut [u8]> {
        let buf = Self::version().serialize(buf)?;
        let buf = self.peer_info.serialize(buf)?;
        let buf = self.call_sequence.serialize(buf)?;
        let buf = self.call_time.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for CalledReq {
    fn deserialize<'de>(buf: &'de [u8]) -> near_base::NearResult<(Self, &'de [u8])> {
        let (v, buf) = u8::deserialize(buf)?;

        if v != Self::version() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("unmatch version: got:{}, expr:{}", Self::version(), v)));
        }

        let (peer_info, buf) = DeviceObject::deserialize(buf)?;
        let (call_sequence, buf) = u32::deserialize(buf)?;
        let (call_time, buf) = Timestamp::deserialize(buf)?;

        Ok((Self{
            peer_info,
            call_sequence,
            call_time,
        }, buf))
    
    }
}

impl PackageBodyTrait for CalledReq {
    fn version() -> u8 {
        1
    }
}

impl std::fmt::Display for CalledReq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "CalledReq: peer_id: {}, call_sequence: {}, call_time: {}",
            self.peer_info.object_id(),
            self.call_sequence,
            self.call_time,
        )
    }
}

#[derive(Clone, Default)]
pub struct CalledResp {
    pub result: u8,           //
    pub info: Option<DeviceObject>,
}

impl Serialize for CalledResp {
    fn raw_capacity(&self) -> usize {
        Self::version().raw_capacity() +
        self.result.raw_capacity() +
        self.info.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> near_base::NearResult<&'a mut [u8]> {
        let buf = Self::version().serialize(buf)?;
        let buf = self.result.serialize(buf)?;
        let buf = self.info.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for CalledResp {
    fn deserialize<'de>(buf: &'de [u8]) -> near_base::NearResult<(Self, &'de [u8])> {
        let (v, buf) = u8::deserialize(buf)?;

        if v != Self::version() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, format!("unmatch version: got:{}, expr:{}", Self::version(), v)));
        }

        let (result, buf) = u8::deserialize(buf)?;
        let (info, buf) = Option::<DeviceObject>::deserialize(buf)?;

        Ok((Self{
            result,
            info,
        }, buf))
    
    }
}

impl PackageBodyTrait for CalledResp {
    fn version() -> u8 {
        1
    }
}

impl std::fmt::Display for CalledResp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "CalledResp: result: {}, have_info: {}",
            self.result,
            self.info.is_some()
        )
    }
}
