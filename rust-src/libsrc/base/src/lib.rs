
mod message;
mod chunk_v0;
mod module;

pub mod raw_object;

pub use message::{MessageExpire, MessageType, SubscribeMessage, DissubcribeMessage, };
pub use module::ModuleTrait;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
pub enum Command {
    Unknown             = 0,
    SubscribeMessage    = 1,
    DissubcribeMessage  = 2,
}

impl Command {
    pub fn into_value(&self) -> u16 {
        match self {
            Self::SubscribeMessage      => 1,
            Self::DissubcribeMessage    => 2,
            _                           => 0,
        }
    }
}

impl From<u16> for Command {
    fn from(v: u16) -> Self {
        match v {
            1   => Self::SubscribeMessage,
            2   => Self::DissubcribeMessage,
            _   => Self::Unknown,
        }
    }

}
