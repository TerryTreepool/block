
pub mod protocol;

pub mod c;
pub mod s;

pub static GROUP_HOST: std::net::Ipv4Addr = std::net::Ipv4Addr::new(224, 1, 2, 3);
pub static GROUP_PORT: u16 = 36912;
