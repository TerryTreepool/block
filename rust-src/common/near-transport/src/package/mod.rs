
pub mod package_builder;
pub mod package_parser;
pub mod package_header;
pub mod package;
pub mod body;
pub mod package_decode;

mod any;

pub use package_builder::*;
pub use package_parser::*;
pub use package_header::{PackageHeader, PackageHeaderExt};
pub use package::{PackageBodyTrait, DynamicPackage, };
pub use body::*;

pub use any::AnyNamedRequest;

use near_base::{Serialize, Deserialize, 
                NearResult, NearError, ErrorCode, 
    };

// #[derive(Clone, Copy)]
// pub enum CallSubCommand {
//     Call,
//     CallResp,
//     Called,
//     CalledResp,
// }

// impl FromStr for CallSubCommand {
//     type Err = NearError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "call" => Ok(Self::Call),
//             "call-resp" => Ok(Self::CallResp),
//             "called" => Ok(Self::Called),
//             "called-resp" => Ok(Self::CalledResp),
//             _ => Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, format!("{s} was undefined."))),
//         }
//     }
// }

// impl std::fmt::Display for CallSubCommand {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Call          => write!(f, "call"),
//             Self::CallResp      => write!(f, "call-resp"),
//             Self::Called        => write!(f, "called"),
//             Self::CalledResp    => write!(f, "called-resp"),
//         }
//     }
// }

#[derive(Clone, Copy)]
pub enum MajorCommand {
    None,
    Exchange,
    AckAckTunnel,
    AckTunnel,
    Ack,
    AckAck,
    Stun,
    // Ping,
    // PingResp,
    // CallCommand,

    Request,
    Response,
}

impl MajorCommand {
    pub fn into_value(&self) -> u8 {
        match self {
            Self::None          => 0x0,
            Self::Exchange      => 0x1,
            Self::AckAckTunnel  => 0x2,
            Self::AckTunnel     => 0x3,
            Self::Stun          => 0x4,
            // Self::Ping          => 0x4,
            // Self::PingResp      => 0x5,
            Self::Ack           => 0x6,
            Self::AckAck        => 0x7,
            // Self::CallCommand   => 0x8,
            Self::Request       => 0xe,
            Self::Response      => 0xf,
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            Self::None          => "None",
            Self::Exchange      => "Exchange",
            Self::AckAckTunnel  => "AckAckTunnel",
            Self::AckTunnel     => "AckTunnel",
            Self::Stun          => "Stun",
            // Self::Ping          => "Ping",
            // Self::PingResp      => "PingResp",
            Self::Ack           => "Ack",
            Self::AckAck        => "AckAck",
            // Self::CallCommand   => "CallCommand",
            Self::Request       => "Request",
            Self::Response      => "Response",
        }
    }
}

impl std::fmt::Display for MajorCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl TryInto<MajorCommand> for u8 {
    type Error = NearError;

    fn try_into(self) -> Result<MajorCommand, Self::Error> {
        match self {
            0x1 => Ok(MajorCommand::Exchange),
            0x2 => Ok(MajorCommand::AckAckTunnel),
            0x3 => Ok(MajorCommand::AckTunnel),
            0x4 => Ok(MajorCommand::Stun),
            // 0x4 => Ok(MajorCommand::Ping),
            // 0x5 => Ok(MajorCommand::PingResp),
            0x6 => Ok(MajorCommand::Ack),
            0x7 => Ok(MajorCommand::AckAck),
            // 0x8 => Ok(MajorCommand::CallCommand),
            0xe => Ok(MajorCommand::Request),
            0xf => Ok(MajorCommand::Response),
            _  => Err(NearError::new(ErrorCode::NEAR_COMMAND_MAJOR, "undefined major")),
        }        
    }
}

impl Serialize for MajorCommand {
    fn raw_capacity(&self) -> usize {
        self.into_value().raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        self.into_value().serialize(buf)
    }
}

impl Deserialize for MajorCommand {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (major, buf) = u8::deserialize(buf)?;
        let major: MajorCommand = u8::try_into(major)?;

        Ok((major, buf))
    }
}

#[derive(Clone)]
pub struct Command {
    major_command: MajorCommand,
    minor_command: Option<String>,
}

impl Command {
    #[inline]
    #[allow(unused)]
    pub fn major_command_value(&self) -> u8 {
        self.major_command.into_value()
    }

    #[inline]
    #[allow(unused)]
    pub fn major_command(&self) -> MajorCommand {
        self.major_command.clone()
    }

    #[inline]
    #[allow(unused)]
    pub fn minor_command<'a>(&'a self) -> Option<&'a str> {
        match self.major_command {
            MajorCommand::Request => self.minor_command.as_ref().map(| data | data.as_str() ),
            _ => None,
        }
    }

}

impl TryFrom<u8> for Command {
    type Error = NearError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let major_command: MajorCommand = u8::try_into(value)?;
        Command::try_from(major_command)
    }

}

impl TryFrom<MajorCommand> for Command {
    type Error = NearError;

    fn try_from(major_command: MajorCommand) -> Result<Self, Self::Error> {
        match major_command {
            MajorCommand::Exchange | MajorCommand::AckTunnel | MajorCommand::AckAckTunnel | MajorCommand::Stun => Ok(Command { major_command, minor_command: None }),
            MajorCommand::Request | MajorCommand::Response => Err(NearError::new(ErrorCode::NEAR_COMMAND_MINOR, "need minor")),
            _  => Err(NearError::new(ErrorCode::NEAR_COMMAND_MAJOR, "undefined major")),
        }
    }

}

// impl TryFrom<(MajorCommand, CallSubCommand)> for Command {
//     type Error = NearError;

//     fn try_from(context: (MajorCommand, CallSubCommand)) -> Result<Self, Self::Error> {
//         let (major, minor) = context;

//         match major {
//             MajorCommand::CallCommand => Ok({
//                 Command { major_command: major, minor_command: Some(minor.to_string()) }
//             }),
//             _ => Err(NearError::new(ErrorCode::NEAR_COMMAND_MAJOR, "undefined major")),
//         }
//     }
// }

impl TryFrom<(MajorCommand, String)> for Command {
    type Error = NearError;

    fn try_from(context: (MajorCommand, String)) -> Result<Self, Self::Error> {
        let (major, minor) = context;

        match major {
            MajorCommand::Request | MajorCommand::Response => Ok(Command { major_command: major, minor_command: Some(minor) }),
            _ => Err(NearError::new(ErrorCode::NEAR_COMMAND_MAJOR, "undefined major")),
        }
    }
}

impl TryFrom<(u8, String)> for Command {
    type Error = NearError;

    fn try_from(context: (u8, String)) -> Result<Self, Self::Error> {
        let (major, minor) = context;
        let major: MajorCommand = u8::try_into(major)?;

        Command::try_from((major, minor))
    }
}

impl Serialize for Command {
    fn raw_capacity(&self) -> usize {
        self.major_command.raw_capacity() + self.minor_command.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.major_command.serialize(buf)?;
        let buf = self.minor_command.serialize(buf)?;

        Ok(buf)
    }

}

impl Deserialize for Command {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (major, buf) = u8::deserialize(buf)?;
        let (minor, buf) = Option::<String>::deserialize(buf)?;

        let command = if let Some(minor) = minor {
            Command::try_from((major, minor))
        } else {
            Command::try_from(major)
        }?;

        Ok((command, buf))
    }
}
