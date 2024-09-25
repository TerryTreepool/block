
pub mod package_builder;
pub mod package_parser;
pub mod package_header;
pub mod package;
pub mod body;

pub use package_builder::*;
pub use package_parser::*;
pub use package_header::{PackageHeader, PackageHeaderExt};
pub use package::{PackageBodyTrait, DynamicPackage};
pub use body::*;

use near_base::{Serialize, Deserialize, utils::{make_long, unmake_long}, NearResult};

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum Command {
    None,
    Exchange,
    AckAck,
    Ack,

    Custom(u16), 
}

impl Command {
    pub fn into_custom(&self) -> u16 {
        match self {
            Self::Custom(v) => *v,
            _ => 0
        }
    }
}

impl Into<u32> for Command {
    fn into(self) -> u32 {
        let (h, l) = {
            match self {
                Self::Exchange  => (0u16, 1u16),
                Self::AckAck    => (0u16, 2u16),
                Self::Ack       => (0u16, 3u16),
                Self::Custom(v) => (1u16, v),
                _ => unreachable!()
            }
        };

        make_long(h, l)
    }
}

impl Into<Command> for u32 {
    fn into(self) -> Command {
        let (h, l) = unmake_long(self);
        match h {
            0u16 => {
                match l {
                    1 => Command::Exchange,
                    2 => Command::AckAck,
                    3 => Command::Ack,
                    _ => Command::None,
                }
            }
            1u16 => Command::Custom(l),
            _ => Command::None
        }
    }
}

impl Serialize for Command {
    fn raw_capacity(&self) -> usize {
        std::mem::size_of::<u32>()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let c: u32 = self.clone().into();
        c.serialize(buf)
    }

}

impl Deserialize for Command {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (v, buf) = u32::deserialize(buf)?;

        Ok((v.into(), buf))
    }
}
