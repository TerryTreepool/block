
pub mod brand_types;
pub mod hci_types;
pub mod thing_data;


use near_base::{NearError, ErrorCode};

#[derive(Default)]
pub enum Status {
    #[default]
    Eanbled = 1,
    Disabled = 2,
}

impl TryFrom<u32> for Status {
    type Error = NearError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Eanbled),
            2 => Ok(Self::Disabled),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, format!("Unknown status, except = {value}")))
        }
    }
}

impl From<Status> for u32 {
    fn from(s: Status) -> Self {
        match s {
            Status::Eanbled => 1,
            Status::Disabled => 2,
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eanbled => write!(f, "enabled"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}
