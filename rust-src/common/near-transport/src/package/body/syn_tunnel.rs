
use crate::package::{Command, PackageBodyTrait};

use near_base::*;
use near_base::DeviceObject;

pub struct SynTunnel {
    from_device_id: ObjectId,
    to_device_id: ObjectId,
    // TODO: pub from_container_id: IncreaseId,
    from_device_desc: DeviceObject,
    send_time: u64,
}

// impl TryFrom<PackageHeader> for SynTunnel {
//     type Error = NearError;

//     fn try_from(header: PackageHeader) -> Result<Self, Self::Error> {
//         if header.command() != Command::SynTunnel.into() {
//             unreachable!("try convert from {} package", header.command());
//         }

//         Ok(Self{
//             header: header
//         })
//     }

// }

#[async_trait::async_trait]
impl PackageBodyTrait for SynTunnel {
    fn command(&self) -> u8 {
        Command::SynTunnel.into()
    }
}

impl Serialize for SynTunnel {
    fn raw_capacity(&self) -> usize {
        0
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.from_device_id.serialize(buf)?;
        let buf = self.to_device_id.serialize(buf)?;
        let buf = self.from_device_desc.serialize(buf)?;
        let buf = self.send_time.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for SynTunnel {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])>
    {
        let (from_device_id, buf) = ObjectId::deserialize(buf)?;
        let (to_device_id, buf) = ObjectId::deserialize(buf)?;
        let (from_device_desc, buf) = DeviceObject::deserialize(buf)?;
        let (send_time, buf) = u64::deserialize(buf)?;

        Ok((Self{
            from_device_id, to_device_id, from_device_desc, send_time
        }, buf))
    }

}
