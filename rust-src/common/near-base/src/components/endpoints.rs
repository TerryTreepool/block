
use std::{net::{SocketAddr, 
                SocketAddrV4, SocketAddrV6, 
                Ipv4Addr, Ipv6Addr}, 
          str::FromStr};

use crate::errors::{ErrorCode, NearError, NearResult};
use crate::codec::{Serialize, Deserialize};

const PROTOCOL_TCP_NAME: &str = "Tcp";
const PROTOCOL_UDP_NAME: &str = "Udp";

const ENDPOINT_PROTOCOL_TCP: u32 = 1u32 << 1;
const ENDPOINT_PROTOCOL_UDP: u32 = 1u32 << 2;

const ENDPOINT_IP_VERSION_4: u32 = 1u32 << 4;
const ENDPOINT_IP_VERSION_6: u32 = 1u32 << 5;

const ENDPOINT_FLAG_DEFAULT: u32 = 1u32 << 7;
const ENDPOINT_FLAG_STATIC_WAN: u32 = 1u32 << 8;
const ENDPOINT_FLAG_LOCAL: u32 = 1u32 << 9;

const IPV4_CAPACITY: usize = 4;
const IPV6_CAPACITY: usize = 16;

impl Serialize for &SocketAddr {
    fn raw_capacity(&self) -> usize {
        match self {
            SocketAddr::V4(addr) => 
                addr.ip().octets().raw_capacity() + addr.port().raw_capacity(),
            SocketAddr::V6(addr) => 
                addr.ip().octets().raw_capacity() + addr.port().raw_capacity(),
        }
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        match self {
            SocketAddr::V4(addr) => {
                let buf = addr.ip().octets().serialize(buf)?;
                let buf = addr.port().serialize(buf)?;
                Ok(buf)
            },
            SocketAddr::V6(addr) => {
                let buf = addr.ip().octets().serialize(buf)?;
                let buf = addr.port().serialize(buf)?;
                Ok(buf)
            },
        }
    }
}

impl Deserialize for SocketAddr {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = Vec::<u8>::deserialize(buf)?;

        match v.len() {
            IPV4_CAPACITY => {
                let (port, buf) = u16::deserialize(buf)?;

                Ok((SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(v[0], v[1], v[2], v[3]), port)), buf))
            }
            IPV6_CAPACITY => {
                let (port, buf) = u16::deserialize(buf)?;

                Ok((SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::from({
                    let mut addr = [0u8; IPV6_CAPACITY];
                    addr.copy_from_slice(v.as_slice());
                    addr
                }), port, 0, 0)), buf))
            }
            _ => { unreachable!() }
        }


    //     let (port, buf) = u16::deserialize(buf)?;

    //     (Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(v[0], v[1], v[2], v[3]), port))), buf)
    // } else if (flag & ENDPOINT_IP_VERSION_6) != 0 {
    //     let (v, buf) = Vec::<u8>::deserialize(buf)?;

    //     if v.len() != IPV6_CAPACITY {
    //         return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, 
    //                            format!("failed deserialize ipv6-addr with expcet={}, got={}", v.len(), IPV6_CAPACITY)));
    //     }

    //     let (port, buf) = u16::deserialize(buf)?;

    //     (Some(SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::from({
    //         let mut addr = [0u8; IPV6_CAPACITY];
    //         addr.copy_from_slice(v.as_slice());
    //         addr
    //     }), port, 0, 0))), buf)

    }

}

pub enum ProtocolType {
    Tcp = 1,
    Udp = 2,
}

#[derive(Clone, Eq, Hash)]
enum Protocol {
    Tcp(SocketAddr),
    Udp(SocketAddr),
}

impl Protocol {
    pub fn to_sockaddr<'a>(&'a self) -> &'a SocketAddr {
        match &self {
            Protocol::Tcp(addr) => { addr },
            Protocol::Udp(addr) => { addr },
        }
    }

    pub fn mut_sockaddr<'a>(&'a mut self) -> Option<&'a mut SocketAddr> {
        match self {
            Protocol::Tcp(addr) => { Some(addr) },
            Protocol::Udp(addr) => { Some(addr) },
        }
    }

    pub fn _capacity(&self) -> usize {
        match &self {
            Self::Tcp(addr) | Self::Udp(addr) => {
                match addr {
                    SocketAddr::V4(_) => { std::mem::size_of::<u16>() + IPV4_CAPACITY },
                    SocketAddr::V6(_) => { std::mem::size_of::<u16>() + IPV6_CAPACITY },
                }
            }
        }
    }

}

impl PartialEq for Protocol {
    fn eq(&self, other: &Protocol) -> bool {
        let src = self.to_sockaddr();
        let dst = other.to_sockaddr();

        src == dst
    }
}

impl std::cmp::PartialOrd for Protocol {
    fn partial_cmp(&self, other: &Protocol) -> Option<std::cmp::Ordering> {
        fn analyze_protocol<'a>(protocol: &'a Protocol) -> (u8, Option<&'a SocketAddr>) {
            match protocol {
                Protocol::Tcp(addr) => { (1u8, Some(addr)) }
                Protocol::Udp(addr) => { (2u8, Some(addr)) }
            }
        }

        let my_protocol = analyze_protocol(self);
        let ot_protocol = analyze_protocol(other);

        match my_protocol.0.cmp(&ot_protocol.0) {
            std::cmp::Ordering::Equal => {
                if let Some(my) = my_protocol.1 {
                    if let Some(ot) = ot_protocol.1 {
                        my.partial_cmp(ot)
                    } else {
                        Some(std::cmp::Ordering::Greater)
                    }
                } else {
                    if let Some(_) = ot_protocol.1 {
                        Some(std::cmp::Ordering::Less)
                    } else {
                        Some(std::cmp::Ordering::Equal)
                    }
                }
            },
            std::cmp::Ordering::Greater => { Some(std::cmp::Ordering::Greater) },
            std::cmp::Ordering::Less => { Some(std::cmp::Ordering::Less) },
        }
    }
}

#[derive(Clone, Eq, Hash)]
pub struct Endpoint {
    flag: u32,
    protocol: Protocol,
}

// impl std::default::Default for Endpoint {
//     fn default() -> Self {
//         Self {
//             flag: 0,
//             protocol: Protocol::Unk,
//         }
//     }
// }

impl Endpoint {
    pub fn default_tcp(addr: SocketAddr) -> Self {
        let flag = match addr {
            SocketAddr::V4(_) => { ENDPOINT_FLAG_DEFAULT | ENDPOINT_IP_VERSION_4 | ENDPOINT_PROTOCOL_TCP },
            SocketAddr::V6(_) => { ENDPOINT_FLAG_DEFAULT | ENDPOINT_IP_VERSION_6 | ENDPOINT_PROTOCOL_TCP },
        };

        Self {
            flag: flag as u32,
            protocol: Protocol::Tcp(addr),
        }
    }

    pub fn default_udp(addr: SocketAddr) -> Self {
        let flag = match addr {
            SocketAddr::V4(_) => { ENDPOINT_FLAG_DEFAULT | ENDPOINT_IP_VERSION_4 | ENDPOINT_PROTOCOL_UDP },
            SocketAddr::V6(_) => { ENDPOINT_FLAG_DEFAULT | ENDPOINT_IP_VERSION_6 | ENDPOINT_PROTOCOL_UDP },
        };

        Self {
            flag: flag as u32,
            protocol: Protocol::Udp(addr),
        }
    }

    #[inline]
    pub fn addr<'a>(&'a self) -> &'a SocketAddr {
        self.protocol.to_sockaddr()
    }

    #[inline]
    pub fn mut_addr<'a>(&'a mut self) -> Option<&'a mut SocketAddr> {
        self.protocol.mut_sockaddr()
    }

    #[inline]
    pub fn is_ipv4(&self) -> bool {
        self.flag & ENDPOINT_IP_VERSION_4 != 0
    }

    #[inline]
    pub fn is_ipv6(&self) -> bool {
        self.flag & ENDPOINT_IP_VERSION_6 != 0
    }

    #[inline]
    pub fn is_udp(&self) -> bool {
        self.flag & ENDPOINT_PROTOCOL_UDP != 0
    }

    #[inline]
    pub fn is_tcp(&self) -> bool {
        self.flag & ENDPOINT_PROTOCOL_TCP != 0
    }

    #[inline]
    pub fn is_sys_default(&self) -> bool {
        self.flag & ENDPOINT_FLAG_DEFAULT != 0
    }

    #[inline]
    pub fn is_static_wan(&self) -> bool {
        self.flag & ENDPOINT_FLAG_STATIC_WAN != 0
    }

    #[inline]
    pub fn set_sys_default(mut self, is_sys_default: bool) -> Self {
        if is_sys_default {
            self.flag &= !(ENDPOINT_FLAG_DEFAULT | ENDPOINT_FLAG_STATIC_WAN | ENDPOINT_FLAG_LOCAL);
            self.flag |= ENDPOINT_FLAG_DEFAULT;
        } else {
            self.flag &= !ENDPOINT_FLAG_DEFAULT;
        }
        self
    }

    #[inline]
    pub fn set_static_wan(mut self, is_static_wan: bool) -> Self {
        if is_static_wan {
            self.flag &= !(ENDPOINT_FLAG_DEFAULT | ENDPOINT_FLAG_STATIC_WAN | ENDPOINT_FLAG_LOCAL);
            self.flag |= ENDPOINT_FLAG_STATIC_WAN;
        } else {
            self.flag &= !ENDPOINT_FLAG_STATIC_WAN;
        }
        self
    }
}

impl std::cmp::PartialOrd for Endpoint {
    fn partial_cmp(&self, other: &Endpoint) -> Option<std::cmp::Ordering> {
        self.protocol.partial_cmp(&other.protocol)
    }
}

impl std::cmp::Ord for Endpoint {
    fn cmp(&self, other: &Endpoint) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Endpoint {
    fn eq(&self, other: &Endpoint) -> bool {
        self.flag == other.flag && self.protocol == other.protocol
    }
}

impl std::net::ToSocketAddrs for Endpoint {
    type Iter = <SocketAddr as std::net::ToSocketAddrs>::Iter;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        self.addr().to_socket_addrs()
    }

}

impl From<(ProtocolType, SocketAddr)> for Endpoint {
    fn from(param: (ProtocolType, SocketAddr)) -> Self {
        match param.0 {
            ProtocolType::Tcp => {
                Endpoint::default_tcp(param.1)
            },
            ProtocolType::Udp => {
                Endpoint::default_udp(param.1)
            }
        }
    }

}

impl FromStr for Endpoint {
    type Err = NearError;

    fn from_str(ep: &str) -> Result<Self, Self::Err> {
        let mut ep_flag = 0;

        // addr type
        let remain_str = match &ep[0..1] {
            "W" => { ep_flag |= ENDPOINT_FLAG_STATIC_WAN; &ep[1..] },
            "L" => { ep_flag |= ENDPOINT_FLAG_LOCAL; &ep[1..] },
            "D" => { ep_flag |= ENDPOINT_FLAG_DEFAULT; &ep[1..] },
            _ => { return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("invalid endpoint string [{}]", ep))); }
        };

        // addr version
        let remain_str = match &remain_str[0..1] {
            "4" => { ep_flag |= ENDPOINT_IP_VERSION_4; &remain_str[1..] },
            "6" => { ep_flag |= ENDPOINT_IP_VERSION_6; &remain_str[1..] },
            _ => { return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("invalid endpoint string [{}]", ep))); }
        };

        // addr protocol
        let remain_str = match &remain_str[0..3] {
            PROTOCOL_TCP_NAME => { ep_flag |= ENDPOINT_PROTOCOL_TCP; &remain_str[3..] },
            PROTOCOL_UDP_NAME => { ep_flag |= ENDPOINT_PROTOCOL_UDP; &remain_str[3..] },
            _ => { return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("invalid endpoint string [{}]", ep))); }
        };

        // addr endpoint
        let addr = SocketAddr::from_str(remain_str).map_err(|_err| {
                                                                        NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("invalid endpoint string [{}]", ep))
                                                                        })?;

        if !((addr.is_ipv4() && ep_flag & ENDPOINT_IP_VERSION_4 != 0) ||
             (addr.is_ipv6() && ep_flag & ENDPOINT_IP_VERSION_6 != 0)) {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("invalid endpoint string [{}]", ep)));
           }

        Ok(Self{
            flag: ep_flag as u32,
            protocol: {
                if ep_flag & ENDPOINT_PROTOCOL_TCP != 0{
                    Protocol::Tcp(addr)
                } else if ep_flag & ENDPOINT_PROTOCOL_UDP != 0 {
                    Protocol::Udp(addr)
                } else {
                    unreachable!("don't reach here.")
                }
            }
        })
    }

}

impl std::fmt::Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ep_type = if (self.flag & ENDPOINT_FLAG_STATIC_WAN) != 0 {
            'W' // wan
        } else if (self.flag & ENDPOINT_FLAG_DEFAULT) != 0 {
            'D' // default
        } else if (self.flag & ENDPOINT_FLAG_LOCAL) != 0 {
            'L' // local
        } else {
            'L'
        };

        let (ep_addr_type, ep_addr_type_str, ep_addr) = match &self.protocol {
            Protocol::Tcp(addr) => {
                if addr.is_ipv4() {
                    (4, PROTOCOL_TCP_NAME, addr.to_string())
                } else {
                    (6, PROTOCOL_TCP_NAME, addr.to_string())
                }
            }
            Protocol::Udp(addr) => {
                if addr.is_ipv4() {
                    (4, PROTOCOL_UDP_NAME, addr.to_string())
                } else {
                    (6, PROTOCOL_UDP_NAME, addr.to_string())
                }
            }
        };

        write!(f, "{}{}{}{}", ep_type, ep_addr_type, ep_addr_type_str, ep_addr)
    }

}

impl std::fmt::Debug for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn std::fmt::Display).fmt(f)
    }
}

impl Serialize for Endpoint {
    fn raw_capacity(&self) -> usize {
        self.flag.raw_capacity() +
        self.protocol.to_sockaddr().raw_capacity()
    }

    fn serialize<'a>(&self, buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.flag.serialize(buf)?;
        let buf = self.protocol.to_sockaddr().serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for Endpoint {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (flag, buf) = u32::deserialize(buf)?;
    
        // ipaddr
        let (protocol, buf) = {
            let (addr, buf) = SocketAddr::deserialize(buf)?;


            if (flag & ENDPOINT_PROTOCOL_TCP) != 0 {
                (Protocol::Tcp(addr), buf)
            } else if (flag & ENDPOINT_PROTOCOL_UDP) != 0 {
                (Protocol::Udp(addr), buf)
            } else {
                unreachable!()
            }

        };

        Ok((Endpoint {
            flag: flag,
            protocol: protocol
        }, buf))
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::components::Endpoint;
    use crate::codec::{Serialize, Deserialize};

    #[test]
    fn test_endpoint() {
        // let ep = Endpoint::default_tcp(SocketAddr::from_str("127.0.0.1:2321").unwrap());
        let ep = Endpoint::from_str("D4Udp127.0.0.1:8899").unwrap();
        let ep_str = ep.to_string();

        let ep_cp = Endpoint::from_str(ep_str.as_str()).unwrap()
                                    .set_static_wan(true);

        let b = ep == ep_cp;
        println!("b={}, ep={}, ep_cp={}", b, ep, ep_cp);

        {
            let mut v = vec![0u8; 1024];

            let _ = ep_cp.serialize(&mut v);

            println!("ep_cp={:?}", v);

            // let mut ep_cp2 = Endpoint::default();

            let (ep_cp2, _) = Endpoint::deserialize(v.as_slice()).unwrap();
        
            println!("ep_cp2={}", ep_cp2);
        }
        // assert_eq!(ep, ep_cp);
    }
}
