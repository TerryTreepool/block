
pub mod process;
// pub mod topic;

mod stack;
mod package;
mod tunnel;
mod network;
mod h;
mod coturn;
mod stack_tunnel_event;
mod finder;

use process::ProcessEventTrait;
pub use stack::Stack;
pub use process::{PackageEventTrait, ProcessTrait, RoutineEventTrait, ItfTrait, ItfBuilderTrait, Routine, RoutineWrap, 
                  EventResult, ResponseEvent, TransferEvent,
        };
pub use network::MTU as PayloadMaxLen;

use std::time::Duration;

use near_base::{DeviceObject, ExtentionObject, people::PeopleObject, ObjectId, NearError, ErrorCode, NearResult, sequence::SequenceString, Endpoint, PrivateKey,
    };
use near_util::Topic;    
    
#[derive(Clone)]
pub struct StackConfig {
    pub polling_interval: Duration,
    pub send_timeout: Duration,
    // pub statistic_interval: Duration, 
    // pub keystore: keystore::Config,
    // pub interface: interface::Config, 
    // pub sn_client: sn::client::Config,
    pub tunnel: tunnel::Config,
    pub peer_c_s: coturn::stun::s::Config,
    pub peer_c_c: coturn::stun::c::Config,
    pub turn_config: coturn::turn::Config,
    // pub turn_c_s: coturn::turn
    // pub stream: stream::Config,
    // pub datagram: datagram::Config,
    // pub ndn: ndn::Config, 
    // pub debug: Option<debug::Config>
}

impl StackConfig {
    pub fn new() -> Self {
        Self {
            polling_interval: Duration::from_millis(100),
            send_timeout: Duration::from_secs(5),
            tunnel: tunnel::Config {
                manager: tunnel::TunnelManagerConfig::default(),
                container: tunnel::TunnelContainerConfig::default(),
                tcp_config: Default::default(),
                udp_config: Default::default(),
            },
            peer_c_s: coturn::stun::s::Config::default(),
            peer_c_c: coturn::stun::c::Config::default(),
            turn_config: coturn::turn::Config::default(),
        }
    }
}

pub struct StackServiceParams {
    pub core_service: DeviceObject,
    pub core_service_private_key: PrivateKey,
    pub sn_service: Vec<DeviceObject>,
    pub service_process_impl: Box<dyn ProcessTrait>,
}

pub struct StackRuntimeParams {
    pub core_service: DeviceObject,
    pub local_extention: ExtentionObject,
    pub runtime_process_impl: Box<dyn ProcessTrait>,
    pub runtime_process_event_impl: Option<Box<dyn ProcessEventTrait>>,

}

pub struct StackPeopleParams {
    pub core_service: DeviceObject,
    pub people: PeopleObject,
    pub people_private_key: PrivateKey,
    pub people_event_impl: Box<dyn ProcessTrait>,
    pub people_process_event_impl: Option<Box<dyn ProcessEventTrait>>,
}

pub struct StackOpenParams {
    // pub stack_type: StackOpenType,
    pub config: Option<StackConfig>,
    pub device_cacher: Option<Box<dyn finder::OuterDeviceCache>>,
}

#[derive(Clone)]
pub enum CommandParam {
    Request(SequenceString),
    Response(SequenceString),
}

impl CommandParam {
    pub fn sequence(&self) -> &SequenceString {
        match self {
            Self::Request(v) => v,
            Self::Response(v) => v,
        }
    }

    pub fn into_sequence(self) -> SequenceString {
        match self {
            Self::Request(v) => v,
            Self::Response(v) => v,
        }
    }

}

impl std::fmt::Display for CommandParam {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(v) => write!(f, "Request(sequence: {v})"),
            Self::Response(v) => write!(f, "Response(sequence: {v})"),
        }
    }
}

pub trait InterfaceMetaTrait: Send + Sync {
    fn clone_as_interface(&self) -> Box<dyn InterfaceMetaTrait>;
    fn local_endpoint(&self) -> Endpoint;
    fn remote_endpoint(&self) -> Endpoint;
}
pub type InterfaceMetaTraitRef = Box<dyn InterfaceMetaTrait>;

impl Clone for InterfaceMetaTraitRef {
    fn clone(&self) -> Self {
        self.as_ref().clone_as_interface()
    }
}

#[derive(Clone, Default)]
pub struct CreatorMeta {
    /// 创建者
    pub creator: Option<ObjectId>,
    /// 创建者local-endpoint
    pub creator_local: Option<Endpoint>,
    /// 创建者remote-endpoint
    pub creator_remote: Option<Endpoint>,
}

impl CreatorMeta {
    pub fn split(self) -> (Option<ObjectId>, Option<Endpoint>, Option<Endpoint>) {
        (self.creator, self.creator_local, self.creator_remote)
    }
}

impl std::fmt::Display for CreatorMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "creator:{:?}, local: {:?}, remote: {:?}", self.creator, self.creator_local, self.creator_remote)
    }
}

impl std::fmt::Debug for CreatorMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn std::fmt::Display).fmt(f)
    }
}

#[derive(Clone)]
pub struct HeaderMeta {
    /// command
    pub command: CommandParam,
    /// 创建者
    pub creator: Option<CreatorMeta>,
    /// 请求者
    pub requestor: ObjectId,
    /// 目的地
    pub to: ObjectId,
    /// 主题
    pub topic: Topic,
    /// 消息时间戳
    pub timestamp: u64,
    /// Net元素
    pub net_meta: Option<InterfaceMetaTraitRef>,
}

impl std::fmt::Display for HeaderMeta {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "command: {}, creator: {:?}, requestor: {}, to: {}, topic: {}, timestamp: {}",
            self.command, self.creator, self.requestor, self.to, self.topic, self.timestamp
        )
    }

}

impl HeaderMeta {
    pub(crate) fn new(head: package::PackageHeader, 
                      head_ext: package::PackageHeaderExt,
                      net_meta: Option<InterfaceMetaTraitRef>) -> NearResult<Self> {

        let timestamp = head.timestamp();
        let (major, sequence) = head.split();

        let command = match major {
            package::MajorCommand::Request => CommandParam::Request(sequence),
            package::MajorCommand::Response => CommandParam::Response(sequence),
            _ => { unreachable!() }
        };

        let (from, to, topic) = head_ext.split();

        let topic = 
            topic.map(| topic | Topic::from(topic))
                .ok_or_else(|| {
                    NearError::new(ErrorCode::NEAR_ERROR_TOPIC_EXCEPTION, "topic is empty")
                })?;
    
        Ok(Self {
            command,
            creator: Some(CreatorMeta{
                creator: from.creator, 
                creator_local: from.creator_local, 
                creator_remote: from.creator_remote
            }),
            requestor: from.requestor,
            to,
            topic,
            timestamp,
            net_meta
        })

    }

    #[inline]
    pub fn sequence(&self) -> &SequenceString {
        self.command.sequence()
    }

}

#[derive(Clone)]
pub struct RequestorMeta {
    pub sequence: Option<SequenceString>,
    pub creator: Option<CreatorMeta>,
    pub requestor: Option<ObjectId>,
    pub to: Option<ObjectId>,
    pub topic: Option<Topic>,
    pub timestamp: Option<u64>,
    pub need_sign: bool,
}

impl std::default::Default for RequestorMeta {
    fn default() -> Self {
        Self {
            sequence: None,
            creator: None,
            requestor: None,
            to: None,
            topic: None,
            timestamp: None,
            need_sign: false,
        }
    }
}

impl std::fmt::Display for RequestorMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sequence: {:?}, requestor: {:?}, target: {:?}",
            self.sequence,
            self.requestor,
            self.to,
        )
    }
}
