
use crate::{utils, };
use crate::errors::{ErrorCode, NearError, NearResult};
use crate::codec::{Serialize, Deserialize};

#[derive(Eq, PartialEq, Clone)]
pub struct Area {
    country: u16,   // 国家编码
    carrier: u8,    // 州，省编码
    city: u16,      // 城镇编码
    device_type: u8,// 设备编码
}

impl Area {
    pub fn new(country: u16, carrier: u8, city: u16, device_type: u8) -> Self {
        Self {
            country,
            carrier,
            city,
            device_type
        }
    }
}

impl std::str::FromStr for Area {
    type Err = NearError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let p = s.split(':');
        let mut array = vec![];
        
        for i in p {
            match u16::from_str(i) {
                Ok(v) => array.push(v),
                Err(_) => { return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "invalid param")); }
            }
        }

        if array.len() != 4 {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "invalid param"));
        }

        Ok(Area::new(array[0],
                     array[1] as u8,
                     array[2],
                     array[3] as u8))

    }

}

impl Into<u64> for Area {
    fn into(self) -> u64 {
        let features_1 = utils::make_long(self.country, self.carrier as u16);
        let features_2 = utils::make_long(self.city, self.device_type as u16);
        let val = utils::make_longlong(features_1, features_2);
        val
    }

}

impl TryFrom<u64> for Area {
    type Error = NearError;

    fn try_from(val: u64) -> Result<Self, Self::Error> {
        let (feature_1, feature_2) = utils::unmake_longlong(val);
        let ((country, carrier), (city, device_type)) = {
            (utils::unmake_long(feature_1), utils::unmake_long(feature_2))
        };

        // TODO
        // 验证国家、地区、城市编码

        Ok(Self{
            country: country, carrier: carrier as u8,
            city: city, device_type: device_type as u8,
        })
    }

}

impl Default for Area {
    fn default() -> Self {
        Self {
            country: 0,
            carrier: 0,
            city: 0,
            device_type: 0u8,
        }
    }
}

impl std::fmt::Debug for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn std::fmt::Display).fmt(f)
    }
}

impl std::fmt::Display for Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
               "{}:{}:{}:{}",
               self.country, self.carrier, self.city,
               self.device_type)
    }
}

impl Serialize for Area {
    fn raw_capacity(&self) -> usize {
        std::mem::size_of::<u64>()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        {
            let val: u64 = self.clone().into();
            val
        }.serialize(buf)
    }

}

impl Deserialize for Area {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (val, buf) = u64::deserialize(buf)?;
        
        Ok((Area::try_from(val)?, buf))
    }

}

#[cfg(test)]
mod test_area {
    use crate::components::DeviceType;
    use crate::components::Area;
    use crate::*;
    
    #[test]
    fn test() {
        use std::str::FromStr;

        let a = Area::new(1234, 1, 5678, DeviceType::Service.into());
        let b = a.to_string();
        println!("test_area: {}", a);
        
        if let Ok(ap) = Area::from_str(b.as_str()) {
            println!("ap = {}", ap);
            assert!(a == ap);
        }

        let mut b = [0u8; 1024];

        let _r = a.serialize(&mut b).unwrap();

        let (a2, _) = Area::deserialize(&b).unwrap();

        println!("test_area2 = {}", a2);

    }
}

