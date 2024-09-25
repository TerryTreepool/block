
pub mod ping;

mod task;

use std::time::Duration;

#[derive(Copy, Clone)]
pub struct Config {
    pub min_random_vport: u16,
    pub max_random_vport: u16,
    pub max_try_random_vport_times: usize,

    pub ping_interval_connect: Duration,
    pub ping_interval: Duration,
    pub offline: Duration,

    pub call_interval: Duration,
    pub call_timeout: Duration,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            min_random_vport: 32767,
            max_random_vport: 65535,
            max_try_random_vport_times: 1,

            ping_interval_connect: Duration::from_secs(30),
            ping_interval: Duration::from_millis(25000),
            offline: Duration::from_secs(120),
            call_interval: Duration::from_millis(200),
            call_timeout: Duration::from_millis(3000),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum NetworkAccessType {
    Unknown,
    NAT,
    Symmetric,
}

impl std::fmt::Display for NetworkAccessType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = 
            match self {
                Self::Unknown => "Unknown",
                Self::NAT => "NAT",
                Self::Symmetric => "Symmetric",
            };

        write!(f, "{name}")
    }
}

impl TryFrom<u8> for NetworkAccessType {
    type Error = near_base::NearError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::NAT),
            2 => Ok(Self::Symmetric),
            _ => Err(near_base::NearError::new(near_base::ErrorCode::NEAR_ERROR_UNDEFINED, format!("{value} undefined")))
        }
    }
}
