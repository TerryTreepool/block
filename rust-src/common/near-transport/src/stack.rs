use log::{debug, error, info, trace, warn};
use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::{Arc, RwLock},
};

use near_base::{any::AnyNamedObject, device::DeviceId, sequence::SequenceString, *};
use near_util::Topic;

use crate::{
    coturn::turn::{TurnService, TurnTask}, finder::DeviceCache, h::OnTimeTrait, package::{MajorCommand, StunReq, StunType}, process::{EmptyProcessEvent, PackageFailureTrait, ProcessEventTrait}, tunnel::PostMessageTrait, TransferEvent
};
use crate::package::{PackageDataSet, PackageHeader, PackageHeaderExt, SequenceBuild};
use crate::process::{
    provider::{EventTextResult, RoutineEventCache},
    EventManager,
};
use crate::coturn::stun::c::ping::PingManager;
use crate::coturn::stun::s::peer_manager::PeerManager;
use crate::stack_tunnel_event::{RuntimeTunnelEvent, ServiceTunnelEvent};
use crate::tunnel::{tunnel::State, TunnelEventTrait};
use crate::{
    h::OnBuildPackage,
    network::{DataContext, NetManager, MTU},
    package::{AnyNamedRequest, PackageBuilder},
};
use crate::{CommandParam, HeaderMeta, ItfBuilderTrait, RoutineEventTrait, StackPeopleParams};
use crate::{CreatorMeta, InterfaceMetaTrait, RequestorMeta};

use super::{
    network::{TcpInterface, TcpPackageEventTrait, UdpInterface, UdpPackageEventTrait},
    process::{PackageEstablishedTrait, PackageEventTrait, ProcessTrait},
    tunnel::{DynamicTunnel, TunnelManager},
    StackConfig, StackOpenParams, StackRuntimeParams, StackServiceParams,
};

enum CoturnState {
    None,
    C(CoturnClient),
    S(CoturnServer),
    // C(PingManager),
    // S(PeerManager),
}

struct CoturnServer {
    stun_server: PeerManager,
    turn_server: TurnService,
}

struct CoturnClient {
    stun_client: PingManager,
    turn_cilent: TurnTask,
}

struct StackComponents {
    cacher_manager: Option<DeviceCache>,
    tunnel_manager: TunnelManager,
    #[allow(unused)]
    net_manager: Option<NetManager>,
    event_manager: EventManager,

    coturn_state: Option<CoturnState>,
    // turn_stub_cachers: Option<ProxyStubCaches>,

    // tunnel_event
    tunnel_event: Box<dyn TunnelEventTrait>,
}

struct StackEvents {
    process_impl: Box<dyn ProcessTrait>,
    process_event_impl: Box<dyn ProcessEventTrait>,
}

enum StackDevice {
    CoreService(StackServiceParams),
    CoturnMiner(StackServiceParams),
    // PnMiner(StackServiceParams),
    Runtime(StackRuntimeParams),
    People(StackPeopleParams),
}

struct StackImpl {
    local: StackDevice,
    local_device_id: ObjectId,
    local_random: Sequence,

    config: StackConfig,
    // aes key
    aes_key: RwLock<AesKey>,

    components: Option<StackComponents>,

    events: StackEvents,
}

#[derive(Clone)]
pub struct Stack(Arc<StackImpl>);

impl Stack {
    pub async fn open_service(
        stack_device: StackServiceParams,
        params: StackOpenParams,
    ) -> NearResult<Self> {
        enum ServiceCodec {
            CoreService,
            CoturnMiner,
        }

        let sn_service = stack_device.sn_service.clone();

        let process_impl = stack_device.service_process_impl.clone_as_process();

        let local_device_id = stack_device.core_service.object_id().clone();
        let local_device = stack_device.core_service.clone();
        // check device codec
        let local_service_codec = {
            let local_service_codec = local_device_id.object_type_code()?;
            match local_service_codec {
                ObjectTypeCode::Device(codec) => {
                    if codec == DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8 {
                        Ok(ServiceCodec::CoreService)
                    } else {
                        let error_string = format!("undefined {codec} device-sub-code");
                        error!("{error_string}");
                        Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, error_string))
                    }
                }
                ObjectTypeCode::Service(codec) => {
                    if codec == ServiceObjectSubCode::OBJECT_TYPE_SERVICE_COTURN_MINER as u8 {
                        Ok(ServiceCodec::CoturnMiner)
                    } else {
                        let error_string = format!("undefined {codec} device-sub-code");
                        error!("{error_string}");
                        Err(NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, error_string))
                    }
                }
                _ => Err(NearError::new(
                    ErrorCode::NEAR_ERROR_INVALIDPARAM,
                    format!("invalid device codec {local_service_codec}"),
                )),
            }
        }?;

        let local_endpoints = {
            let mut local_endpoints = 
                stack_device.core_service
                    .body()
                    .content()
                    .endpoints()
                    .clone();

            local_endpoints.iter_mut().for_each(|ep| {
                if let Some(sockaddr) = ep.mut_addr() {
                    match sockaddr {
                        SocketAddr::V4(v) => v.set_ip(Ipv4Addr::new(0, 0, 0, 0)),
                        SocketAddr::V6(v) => v.set_ip(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)),
                    }
                }
            });

            local_endpoints
        };

        let stack_impl = Arc::new(StackImpl {
            local: {
                match &local_service_codec {
                    ServiceCodec::CoreService => StackDevice::CoreService(stack_device),
                    ServiceCodec::CoturnMiner => StackDevice::CoturnMiner(stack_device),
                    // ServiceCodec::PnMiner => StackDevice::PnMiner(stack_device),
                }
            },
            local_device_id: local_device_id,
            local_random: Sequence::random(),
            config: params.config.unwrap_or(StackConfig::new()),
            aes_key: RwLock::new(AesKey::generate()),
            components: None,
            events: StackEvents {
                process_impl: process_impl,
                process_event_impl: Box::new(EmptyProcessEvent),
            },
        });
        let stack = Self(stack_impl.clone());

        let components = StackComponents {
            cacher_manager: {
                if let Some(device_cacher) = params.device_cacher {
                    Some(DeviceCache::new(local_device).set_outer(device_cacher))
                } else {
                    Some(DeviceCache::new(local_device).set_inner())
                }
            },
            tunnel_manager: TunnelManager::open(stack.clone())?,
            net_manager: None,
            event_manager: EventManager::new(),
            coturn_state: None,
            // turn_stub_cachers: {
            //     match &local_service_codec {
            //         ServiceCodec::CoreService => Some(ProxyStubCaches::default()),
            //         // ServiceCodec::PnMiner => Some(ProxyStubCaches::default()),
            //         ServiceCodec::CoturnMiner => None,
            //     }
            // },
            tunnel_event: Box::new(ServiceTunnelEvent::new(stack.clone())),
        };

        let mut_stack = unsafe { &mut *(Arc::as_ptr(&stack_impl) as *mut StackImpl) };
        mut_stack.components = Some(components);

        // startup net
        let net_manager = {
            NetManager::listen(stack.clone(), local_endpoints.as_slice()).await?
            // // bind custom ip
            // let ot_endpoints =
            //     local_endpoints.iter()
            //         .filter(| ep | {
            //             if let Some(sockaddr) = ep.addr() {
            //                 sockaddr.port() != CORE_STACK_PORT
            //             } else {
            //                 false
            //             }
            //         })
            //         .collect();

            // let net_manager = NetManager::listen(stack.clone(), ot_endpoints).await?;

            // bind near default port
            // net_manager.bind_interface(
            // &Endpoint::default_tcp(SocketAddr::new(IpAddr::from(Ipv4Addr::new(0, 0, 0, 0)), CORE_STACK_PORT))
            // ).await?;

            // net_manager
        };

        mut_stack.components.as_mut().unwrap().net_manager = Some(net_manager);

        mut_stack.components.as_mut().unwrap().coturn_state = Some({
            match local_service_codec {
                ServiceCodec::CoreService => {
                    let manager = PingManager::init(stack.clone()).await?;
                    for sn in sn_service {
                        manager.add_sn(sn).await.map_err(|e| {
                            error!("failed add-sn with err: {e}");
                            e
                        })?;
                    }
                    CoturnState::C(CoturnClient {
                        stun_client: manager,
                        turn_cilent: TurnTask::open(stack.clone())?,
                    })
                }
                ServiceCodec::CoturnMiner => {
                    CoturnState::S(CoturnServer { 
                        stun_server: PeerManager::new(stack.clone()), 
                        turn_server: TurnService::open(
                            stack.clone(), 
                            {
                                if let Some(endpoint) = local_endpoints.get(0) {
                                    Some(endpoint.addr())
                                } else {
                                    None
                                }
                                .cloned()
                            }
                        )?,
                    })
                }
            }
        });

        // start
        stack.start();

        Ok(stack)
    }

    pub async fn open_runtime(
        mut stack_device: StackRuntimeParams,
        params: StackOpenParams,
    ) -> NearResult<Self> {
        // let mut params = params;

        let process_impl = stack_device.runtime_process_impl.clone_as_process();

        let local_device_id = stack_device.local_extention.object_id().clone();
        let core_device = stack_device.core_service.clone();
        let process_event_impl = {
            std::mem::replace(&mut stack_device.runtime_process_event_impl, None)
                .unwrap_or(Box::new(EmptyProcessEvent))
        };

        let stack_impl = Arc::new(StackImpl {
            local: StackDevice::Runtime(stack_device),
            local_device_id: local_device_id,
            local_random: Sequence::random(),
            config: params.config.unwrap_or(StackConfig::new()),
            aes_key: RwLock::new(AesKey::generate()),
            components: None,
            events: StackEvents {
                process_impl: process_impl,
                process_event_impl,
            },
        });
        let stack = Self(stack_impl.clone());

        let mut_stack = unsafe { &mut *(Arc::as_ptr(&stack_impl) as *mut StackImpl) };
        mut_stack.components = Some(StackComponents {
            tunnel_manager: {
                let tunnel_manager = TunnelManager::open(stack.clone())?;
                let _ = tunnel_manager.create_container(core_device.object_id());
                tunnel_manager
            },
            cacher_manager: None,
            net_manager: None,
            coturn_state: Some(CoturnState::None),
            // turn_stub_cachers: None,
            event_manager: EventManager::new(),
            tunnel_event: Box::new(RuntimeTunnelEvent::new(stack.clone())),
        });

        // startup net
        let net_manager = {
            // The externtion internal link. Here, select the ipv4 address of a tcp to connect
            // core endpoint
            // let ep = Endpoint::default_tcp(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), CORE_STACK_PORT));
            let ep_list: Vec<&Endpoint> = stack
                .core_device()
                .body()
                .content()
                .endpoints()
                .iter()
                .filter(|ep| ep.is_tcp())
                .collect();
            //     // let core_ep: Vec<&Endpoint> = stack.core_device()
            //     //                                     .body()
            //     //                                     .content()
            //     //                                     .endpoints()
            //     //                                     .iter()
            //     //                                     .filter(| &ep | {
            //     //                                         ep.is_ipv4()
            //     //                                     })
            //     //                                     .collect();
            //     // if core_ep.len() == 0 {
            //     //     stack.core_device()
            //     //         .body()
            //     //         .content()
            //     //         .endpoints()
            //     //         .iter()
            //     //         .filter(| &ep | {
            //     //             ep.is_ipv6()
            //     //         })
            //     //         .collect()
            //     // } else {
            //     //     core_ep
            //     // }

            // let core_

            if ep_list.len() == 0 {
                Err(NearError::new(
                    ErrorCode::NEAR_ERROR_NOTFOUND,
                    "not found valid network address.",
                ))
            } else {
                NetManager::connect_tcp(stack.clone(), ep_list.get(0).unwrap(), &core_device).await
            }
        }?;

        mut_stack.components.as_mut().unwrap().net_manager = Some(net_manager);
        mut_stack.components.as_mut().unwrap().cacher_manager = Some(DeviceCache::new(core_device).set_inner());

        // start
        stack.start();

        Ok(stack)
    }

    pub async fn open_people(
        mut stack_people: StackPeopleParams,
        params: StackOpenParams,
    ) -> NearResult<Self> {
        let process_impl = stack_people.people_event_impl.clone_as_process();

        let local_device_id = stack_people.people.object_id().clone();
        let core_device = stack_people.core_service.clone();
        let process_event_impl = 
            std::mem::replace(&mut stack_people.people_process_event_impl, None)
                .unwrap_or(Box::new(EmptyProcessEvent));

        let stack_impl = Arc::new(StackImpl {
            local: StackDevice::People(stack_people),
            local_device_id: local_device_id,
            local_random: Sequence::random(),
            config: params.config.unwrap_or(StackConfig::new()),
            aes_key: RwLock::new(AesKey::generate()),
            components: None,
            events: StackEvents {
                process_impl: process_impl,
                process_event_impl
            },
        });
        let stack = Self(stack_impl.clone());

        let mut_stack = unsafe { &mut *(Arc::as_ptr(&stack_impl) as *mut StackImpl) };
        mut_stack.components = Some(StackComponents {
            cacher_manager: None,
            tunnel_manager: {
                let tunnel_manager = TunnelManager::open(stack.clone())?;
                let _ = tunnel_manager.create_container(core_device.object_id());
                tunnel_manager
            },
            net_manager: None,
            coturn_state: Some(CoturnState::None),
            // turn_stub_cachers: None,
            event_manager: EventManager::new(),
            tunnel_event: Box::new(RuntimeTunnelEvent::new(stack.clone())),
        });

        // startup net
        let net_manager = {
            // The externtion internal link. Here, select the ipv4 address of a tcp to connect
            // core endpoint
            // let ep = Endpoint::default_tcp(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), CORE_STACK_PORT));
            let ep_list: Vec<&Endpoint> = {
                stack
                    .core_device()
                    .body()
                    .content()
                    .endpoints()
                    .iter()
                    .filter(|ep| ep.is_tcp())
                    .collect()
            };

            if ep_list.len() == 0 {
                Err(NearError::new(
                    ErrorCode::NEAR_ERROR_NOTFOUND,
                    "not found valid network address.",
                ))
            } else {
                NetManager::connect_tcp(stack.clone(), ep_list.get(0).unwrap(), &core_device).await
            }
        }?;

        mut_stack.components.as_mut().unwrap().net_manager = Some(net_manager);
        mut_stack.components.as_mut().unwrap().cacher_manager = Some(DeviceCache::new(core_device).set_inner());

        // start
        stack.start();

        Ok(stack)
    }

    pub(self) fn start(&self) {
        trace!("on_time_escape");
        let arc_self = self.clone();
        let polling_interval = arc_self.config().polling_interval;

        async_std::task::spawn(async move {
            loop {
                arc_self.tunnel_manager().on_time_escape(now());

                let _ = 
                    async_std::future::timeout(
                        polling_interval,
                        async_std::future::pending::<()>(),
                    )
                    .await;
            }
        });
    }

    pub async fn add_sn(&self, remote: DeviceObject) -> NearResult<()> {
        self.stun_client().add_sn(remote).await
    }

    pub async fn remote_sn(&self, remote_id: &DeviceId) -> NearResult<()> {
        self.stun_client().remove_sn(remote_id).await
    }

    pub fn is_core(&self) -> bool {
        match &self.0.local {
            StackDevice::CoreService(_) | StackDevice::CoturnMiner(_) => true,
            _ => false,
        }
    }

    pub fn local_device_id(&self) -> &ObjectId {
        &self.0.local_device_id
    }

    pub fn local(&self) -> AnyNamedObject {
        match &self.0.local {
            StackDevice::CoreService(params) | 
            StackDevice::CoturnMiner(params) => AnyNamedObject::Device(params.core_service.clone()),
            StackDevice::Runtime(params) => {
                AnyNamedObject::Extention(params.local_extention.clone())
            }
            StackDevice::People(params) => AnyNamedObject::People(params.people.clone()),
        }
    }

    pub fn core_device(&self) -> &DeviceObject {
        match &self.0.local {
            StackDevice::CoreService(c) |
            StackDevice::CoturnMiner(c) => &c.core_service,
            StackDevice::Runtime(v) => &v.core_service,
            StackDevice::People(p) => &p.core_service,
        }
    }

    pub fn config(&self) -> &StackConfig {
        &self.0.config
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn aes_key(&self) -> AesKey {
        self.0.aes_key.read().unwrap().clone()
    }

    #[inline]
    pub(crate) fn tunnel_manager(&self) -> &TunnelManager {
        &self.0.components.as_ref().unwrap().tunnel_manager
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn net_manager(&self) -> &NetManager {
        self.0
            .components
            .as_ref()
            .unwrap()
            .net_manager
            .as_ref()
            .unwrap()
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn cacher_manager(&self) -> &DeviceCache {
        self.0.components.as_ref().unwrap().cacher_manager.as_ref().unwrap()
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn turn_task(&self) -> &TurnTask {
        match &self.0.components.as_ref().unwrap().coturn_state.as_ref().unwrap() {
            CoturnState::None => unreachable!(),
            CoturnState::C(coturn_client) => &coturn_client.turn_cilent,
            CoturnState::S(_) => unreachable!(),
        }
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn stun_client(&self) -> &PingManager {
        match &self.0.components.as_ref().unwrap().coturn_state.as_ref().unwrap() {
            CoturnState::None => unreachable!(),
            CoturnState::C(coturn_client) => &coturn_client.stun_client,
            CoturnState::S(_) => unreachable!(),
        }
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn stun_server(&self) -> &PeerManager {
        match &self.0.components.as_ref().unwrap().coturn_state.as_ref().unwrap() {
            CoturnState::None => unreachable!(),
            CoturnState::C(_) => unreachable!(),
            CoturnState::S(coturn_server) => &coturn_server.stun_server,
        }
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn turn_server(&self) -> &TurnService {
        match &self.0.components.as_ref().unwrap().coturn_state.as_ref().unwrap() {
            CoturnState::None => unreachable!(),
            CoturnState::C(_) => unreachable!(),
            CoturnState::S(coturn_server) => &coturn_server.turn_server,
        }
    }

    // #[inline]
    // #[allow(unused)]
    // pub(crate) fn turn_stub_cachers(&self) -> Option<&ProxyStubCaches> {
    //     self.0.components.as_ref().unwrap().turn_stub_cachers.as_ref()
    // }

    #[inline]
    pub(self) fn event_manager(&self) -> &EventManager {
        &self.0.components.as_ref().unwrap().event_manager
    }

    #[inline]
    pub(self) fn tunnel_event(&self) -> &dyn TunnelEventTrait {
        self.0.components.as_ref().unwrap().tunnel_event.as_ref()
    }

    #[inline]
    pub(self) fn process_impl(&self) -> &dyn ProcessTrait {
        self.0.events.process_impl.as_ref()
    }

    pub fn payload_max_len(&self) -> usize {
        MTU
    }

    #[inline]
    pub(crate) fn as_signer(&self) -> Box<dyn SignerTrait> {
        Box::new(self.clone())
    }
}

impl Stack {
    pub(self) async fn wait_for(&self, remote: &ObjectId) -> bool {
        match self.tunnel_manager().wait_tunnel_active(remote).await {
            State::Established(()) => true,
            _ => false,
        }
    }

    pub async fn wait_online(&self) -> bool {
        self.wait_for(self.core_device().object_id()).await
    }

    pub fn test_online(&self, remote: &ObjectId) -> bool {
        match self.tunnel_manager().test_tunnel_active(remote) {
            State::Established(_) => true,
            _ => false,
        }
    }
}

// #[async_trait::async_trait]
// impl OnBuildPackage<(HeaderMeta, Data), (SequenceString, PackageDataSet)> for Stack {
//     async fn build_package(
//         &self,
//         data_context: (HeaderMeta, Data),
//     ) -> NearResult<(SequenceString, PackageDataSet)> {
//         let (header_meta, body) = data_context;

//         let (sequence, body) = {
//             match header_meta.command {
//                 CommandParam::Request(sequence) => (sequence, AnyNamedRequest::with_request(body)),
//                 CommandParam::Response(sequence) => (sequence, AnyNamedRequest::with_response(body)),
//             }
//         };

//         PackageBuilder::build_head(sequence.clone(), None)
//             .build_topic(
//                 header_meta.creator,
//                 self.local().object_id().clone(),
//                 header_meta.to,
//                 Some(header_meta.topic.into()),
//             )
//             .build_body(body)
//             .build(None)
//             .await
//             .map(|package| (sequence, package))
//             .map_err(|err| {
//                 error!("failed build message with {}", err);
//                 err
//             })

//         // match header_meta.command {
//         //     CommandParam::Request(sequence) => {
//         //         PackageBuilder::build_head(sequence.clone())
//         //             .build_topic(
//         //                 header_meta.creator,
//         //                 self.local().object_id().clone(),
//         //                 header_meta.to,
//         //                 Some(header_meta.topic.into())
//         //             )
//         //             .build_body(AnyNamedRequest::with_request(body))
//         //             .build(None)
//         //             .await
//         //             .map(| package | (sequence, package))
//         //             .map_err(| err | {
//         //                 error!("failed build message with {}", err);
//         //                 err
//         //             })

//         //     }
//         //     CommandParam::Response(sequence) => {
//         //         PackageBuilder::build_head(sequence.clone())
//         //             .build_topic(
//         //                 header_meta.creator,
//         //                 self.local().object_id().clone(),
//         //                 header_meta.to,
//         //                 Some(header_meta.topic.into())
//         //             )
//         //             .build_body(AnyNamedRequest::with_response(body))
//         //             .build(None)
//         //             .await
//         //             .map(| package | (sequence, package))
//         //             .map_err(| err | {
//         //                 error!("failed build message with {}", err);
//         //                 err
//         //             })
//         //     }
//         // }

//         // let (body, sequence) =
//         //     match header_meta.command {
//         //         CommandParam::Request(sequence) => (AnyNamedRequest::with_request(body), sequence),
//         //         CommandParam::Response(sequence) => (AnyNamedRequest::with_response(body), sequence),
//         //     };

//         // PackageBuilder::build_head(sequence.clone())
//         //     .build_topic(
//         //         header_meta.creator,
//         //         self.local().object_id().clone(),
//         //         header_meta.to,
//         //         Some(header_meta.topic.into())
//         //     )
//         //     .build_body(body)
//         //     .build(None)
//         //     .await
//         //     .map(| package | (sequence, package))
//         //     .map_err(| err | {
//         //         error!("failed build message with {}", err);
//         //         err
//         //     })
//     }
// }

///
/// creator: if it isn't existed, default is myself.
/// requestor: default is myself.
/// target: must have it
/// timestamp: default is now()
/// sequence: if it isn't existed, I will use SequenceBuild::build create it.
/// topic: if it isn't existed, I will use AnyNamedRequest::minor_command for get it.
/// need_sign: must have it, true or false.
/// body: must have it.
/// 
#[derive(Default)]
pub(crate) struct BuildPackageV1 {
    pub creator: Option<ObjectId>,
    pub requestor: Option<ObjectId>, 
    pub target: Option<ObjectId>, 
    pub timestamp: Option<Timestamp>, 
    pub sequence: Option<SequenceString>,
    pub topic: Option<String>,
    pub body: AnyNamedRequest,
    pub need_sign: bool,
}

#[async_trait::async_trait]
impl OnBuildPackage<BuildPackageV1, (SequenceString, PackageDataSet)> for Stack
{
    async fn build_package(
        &self,
        data_context: BuildPackageV1,
    ) -> NearResult<(SequenceString, PackageDataSet)> {
        let target = data_context.target.ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "missing target"))?;
        let creator = data_context.creator;
        let requestor = data_context.requestor.unwrap_or(self.local_device_id().clone());
        let timestamp = data_context.timestamp.unwrap_or(now());
        let body = data_context.body;
        let body_name = format!("{body}");
        let need_sign = data_context.need_sign;
        let sequence = 
            match data_context.sequence {
                Some(sequence) => sequence,
                None => {
                    SequenceBuild {
                        requestor: &requestor,
                        now: now(),
                        sync_times: self.0.local_random.generate().into_value(),
                    }
                    .build()?
                }
            };
        let topic = 
            match data_context.topic {
                Some(topic) => Ok(Some(topic)),
                None => {
                    match &body {
                        AnyNamedRequest::None => unreachable!(),
                        AnyNamedRequest::Request(_) | AnyNamedRequest::Response(_) => Err(NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "miss topic item")),
                        _ => Ok(None),
                    }
                }
            }?;

        PackageBuilder::build_head(sequence.clone(), Some(timestamp))
            .build_topic(
                creator.map(| creator | {
                    CreatorMeta {
                        creator: Some(creator),
                        ..Default::default()
                    }
                }),
                requestor,
                target,
                topic,
            )
            .build_body(body)
            .build({
                if need_sign {
                    Some(self.as_signer())
                } else {
                    None
                }
            })
            .await
            .map(|dataset| {
                info!("successfully build {} package to {} sequence {}", body_name, target, sequence);
                (sequence, dataset)
            })
            .map_err(|err| {
                error!("failed build {} package to {} with {}", body_name, target, err);
                err
            })
    }
}

impl Stack {
    pub async fn post_message<B: Serialize + Deserialize>(
        &self,
        requestor_meta: RequestorMeta,
        body: B,
        callback: Option<Box<dyn RoutineEventTrait>>,
    ) -> NearResult<()> {
        let data = {
            let mut data = vec![0u8; body.raw_capacity()];
            body.serialize(data.as_mut_slice())?;
            data
        };

        let this = self.clone();

        let _ = PostMessageTrait::post_message(&this, (requestor_meta, data, callback)).await?;

        Ok(())
    }

    pub async fn post_message_with_builder<BUILD: ItfBuilderTrait>(
        &self,
        requestor_meta: RequestorMeta,
        builder: BUILD,
        callback: Option<Box<dyn RoutineEventTrait>>,
    ) -> NearResult<()> {
        let b: Vec<BUILD::R> = builder.build();
        let this = self.clone();

        struct EventWrapper(Arc<Box<dyn RoutineEventTrait>>);

        #[async_trait::async_trait]
        impl RoutineEventTrait for EventWrapper {
            async fn emit(
                &self,
                header_meta: &HeaderMeta,
                data: Vec<u8>,
            ) -> NearResult<EventTextResult> {
                self.0.emit(header_meta, data).await
            }
        }

        let mut futs = vec![];

        if let Some(cb) = callback {
            let arc_cb = Arc::new(cb);

            for it in b {
                futs.push(
                    this.post_message(
                        requestor_meta.clone(),
                        it,
                        Some(Box::new(EventWrapper(arc_cb.clone()))),
                    )
                );
            }
        } else {
            for it in b {
                futs.push(
                    this.post_message(requestor_meta.clone(), it, None)
                );
            }
        }

        let _ = futures::future::join_all(futs).await;

        Ok(())
    }

}

#[async_trait::async_trait]
impl PostMessageTrait<(RequestorMeta, Vec<u8>, Option<Box<dyn RoutineEventTrait>>)> for Stack {
    type R = SequenceString;
    async fn post_message(
        &self, 
        context: (RequestorMeta, Vec<u8>, Option<Box<dyn RoutineEventTrait>>)
    ) -> NearResult<Self::R> {
        let (requestor_meta, data, callback) = context;
    
        PostMessageTrait::post_message(
            self, (
            requestor_meta, 
            AnyNamedRequest::with_request(data.into()), 
            callback
        ))
        .await
    }
}

#[async_trait::async_trait]
impl PostMessageTrait<(Option<DynamicTunnel>, RequestorMeta, AnyNamedRequest, Option<Box<dyn RoutineEventTrait>>)> for Stack {

    type R = SequenceString;

    async fn post_message(
        &self, 
        context: (Option<DynamicTunnel>, RequestorMeta, AnyNamedRequest, Option<Box<dyn RoutineEventTrait>>)
    ) -> NearResult<Self::R> {
        let (tunnel, requestor_meta, data, callback) = context;

        let this = self.clone();

        let data_name = format!("{data}");

        // if tunnel is exist. target must tunnel peer id.
        let tunnel_target = 
            if let Some(tunnel_ref) = tunnel.as_ref() {
                Ok(tunnel_ref.peer_id())
            } else {
                if self.is_core() {
                    requestor_meta.to.as_ref()
                } else {
                    Some(self.core_device().object_id())
                }
                .ok_or_else(|| {
                    error!("miss target");
                    NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, "miss target.")
                })
            }?
            .clone();
        let tunnel_target_type_codec = 
            tunnel_target.object_type_code().map_err(| err | {
                error!("invalid tunnel target type codec with {}", err);
                err
            })?;

        let creator = 
            requestor_meta
                .creator
                .map(| source | source.creator.unwrap_or(self.local_device_id().clone()))
                .unwrap_or(self.local_device_id().clone());

        let (sequence, package) = 
            self.build_package(BuildPackageV1{
                    creator: Some(creator),
                    requestor: requestor_meta.requestor,
                    target: {
                        if let Some(_) = requestor_meta.to {
                            requestor_meta.to
                        } else {
                            Some(tunnel_target.clone())
                        }
                    },
                    // target: Some(target.clone()),
                    sequence: requestor_meta.sequence,
                    body: data,
                    topic: requestor_meta.topic.map(| topic | topic.into() ),
                    need_sign: requestor_meta.need_sign,
                    ..Default::default()
                })
                .await
                .map_err(| err | {
                    error!("failed build {data_name} package with err: {err}");
                    err
                })?;

        async_std::task::spawn(async move {
            let sequence_ref = &sequence;
            match {
                match tunnel_target_type_codec {
                    ObjectTypeCode::Device(codec) if codec == DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8 => {

                        match &this.0.components.as_ref().unwrap().coturn_state.as_ref().unwrap() {
                            CoturnState::None => {
                                // it's runtime or people
                                if let Some(tunnel) = tunnel {
                                    this.tunnel_manager().post_message((tunnel, sequence_ref.clone(), package)).await
                                } else {
                                    this.tunnel_manager().post_message((tunnel_target, sequence_ref.clone(), package)).await
                                }
                            },
                            CoturnState::C(coturn_client) => {
                                // it's core-service
                                if let Some(tunnel) = tunnel {
                                    this.tunnel_manager().post_message((tunnel, sequence_ref.clone(), package)).await
                                } else {
                                    coturn_client.stun_client.post_message((tunnel_target, sequence_ref.clone(), package)).await
                                }
                            },
                            CoturnState::S(_) => {
                                // // it's coturn-miner
                                if let Some(tunnel) = tunnel {
                                    this.tunnel_manager().post_message((tunnel, sequence_ref.clone(), package)).await
                                } else {
                                    this.tunnel_manager().post_message((tunnel_target, sequence_ref.clone(), package)).await
                                }
                            }
                        }
                    }
                    _ => {
                        this
                            .tunnel_manager()
                            .post_message((tunnel_target, sequence_ref.clone(), package))
                            .await
                    }
                }
            } {
                Ok(_) => {
                    if let Some(callback) = callback {
                        let _ = this.event_manager().join_routine(
                            &tunnel_target,
                            sequence_ref,
                            0,
                            RoutineEventCache::from((this.local_device_id().clone(), callback)),
                        );
                    }
                    info!("successfully post {data_name} package sequence: {}", sequence_ref);
                }
                Err(e) => {
                    error!("Failed to post {data_name} package sequence = {} with err {}", 
                        sequence_ref,
                        e
                    );
                }
            }
        });

        Ok(sequence)
    }
}

#[async_trait::async_trait]
impl PostMessageTrait<(RequestorMeta, AnyNamedRequest, Option<Box<dyn RoutineEventTrait>>)> for Stack {

    type R = SequenceString;

    async fn post_message(
        &self, 
        context: (RequestorMeta, AnyNamedRequest, Option<Box<dyn RoutineEventTrait>>)
    ) -> NearResult<Self::R> {
        let (requestor_meta, data, callback) = context;

        PostMessageTrait::post_message(
            self, (None, requestor_meta, data, callback)
        )
        .await
    }
}

#[async_trait::async_trait]
impl TcpPackageEventTrait for Stack {
    fn on_connected(&self, interface: TcpInterface, remote: &DeviceObject) {
        TcpPackageEventTrait::on_connected(self.tunnel_manager(), interface, remote)
    }

    fn on_closed(&self, interface: &TcpInterface, remote: &ObjectId) {
        self.tunnel_manager().on_closed(interface, remote)
    }

    async fn on_tcp_package(
        &self,
        interface: TcpInterface,
        package: DataContext,
    ) -> NearResult<()> {
        self.tunnel_manager()
            .on_tcp_package(interface, package)
            .await
    }
}

#[async_trait::async_trait]
impl UdpPackageEventTrait<Endpoint> for Stack {
    fn on_connected(
        &self,
        interface: UdpInterface,
        remote: &DeviceObject,
        remote_endpoint: Endpoint,
    ) {
        UdpPackageEventTrait::on_connected(
            self.tunnel_manager(),
            interface,
            remote,
            remote_endpoint,
        )
    }

    async fn on_udp_package(
        &self,
        interface: UdpInterface,
        package: DataContext,
        remote: Endpoint,
    ) -> NearResult<()> {
        self.tunnel_manager()
            .on_udp_package(interface, package, remote)
            .await
    }
}

#[async_trait::async_trait]
impl PackageEventTrait<(StunReq, Signature)> for Stack {
    async fn on_package_event(
        &self,
        tunnel: DynamicTunnel,
        head: PackageHeader,
        head_ext: PackageHeaderExt,
        data: (StunReq, Signature),
    ) -> NearResult<()> {

        let sequence = head.sequence();
        trace!(
            "Stack::PackageEventTrait::Stun::{}: tunnel: {}, head: {}, head_ext: {}, begin...",
            data.0.stun_name(),
            tunnel,
            head, head_ext,
        );

        let stun_resp = 
            match &self.0.components.as_ref().unwrap().coturn_state.as_ref().unwrap() {
                CoturnState::S(coturn_server) => {
                    match data.0.stun_type() {
                        StunType::PingRequest => Some(coturn_server.stun_server.on_bind_request(&head, &head_ext, tunnel.clone(), data).await?),
                        StunType::CallRequest => Some(coturn_server.stun_server.on_call_request(&head, &head_ext, data).await?),
                        StunType::CallResponse | StunType::CallErrorResponse => {
                            coturn_server.stun_server.on_call_response(&head, &head_ext, data).await?;
                            None
                        },
                        StunType::PingResponse | StunType::PingErrorResponse => unreachable!("won't reach here"),
                        StunType::AllocationChannelRequest => {
                            Some(coturn_server.stun_server.on_allocation_request(&head, &head_ext, tunnel.clone(), data).await?)
                        }
                        StunType::AllocationChannelResponse | StunType::AllocationChannelErrorResponse => {
                            coturn_server.stun_server.on_allocation_response(&head, &head_ext, data).await?;
                            None
                        }
                    }
                    // stun_server.on_package_event(tunnel, head, head_ext, data).await
                }
                CoturnState::C(coturn_client) => {
                    match data.0.stun_type() {
                        StunType::PingRequest => Some(coturn_client.stun_client.on_ping_request(&head, &head_ext, data).await?),
                        StunType::PingResponse | StunType::PingErrorResponse => {
                            coturn_client.stun_client.on_ping_response(&head, &head_ext, data).await?;
                            None
                        },
                        StunType::CallRequest => Some(coturn_client.stun_client.on_call_request(&head, &head_ext, data).await?),
                        StunType::CallResponse | StunType::CallErrorResponse => {
                            coturn_client.stun_client.on_call_response(&head, &head_ext, data).await?;
                            None
                        }
                        StunType::AllocationChannelRequest => 
                            Some(coturn_client.stun_client.on_allocation_request(&head, &head_ext, data).await?),
                        StunType::AllocationChannelResponse | StunType::AllocationChannelErrorResponse => {
                            coturn_client.stun_client.on_allocation_response(&head, &head_ext, data).await?;
                            None
                        }
                    }
                }
                CoturnState::None => unreachable!("won't reach here"),
            };

        if let Some(stun_resp) = stun_resp {
            let (source, _, _) = head_ext.split();
            let stun_name = stun_resp.stun_name();
            PostMessageTrait::post_message(
                    self, 
                    (
                        Some(tunnel),
                        RequestorMeta {
                            sequence: Some(sequence.clone()),
                            creator: Some(CreatorMeta {
                                creator: source.creator,
                                ..Default::default()
                            }),
                            need_sign: true,
                            ..Default::default()
                        },
                        AnyNamedRequest::with_stun(stun_resp),
                        None
                    )
                )
                .await
                .map(| _ | {
                    info!("successfully post {stun_name} message sequence: {sequence}", );
                    ()
                })
                .map_err(| err | {
                    error!("failure post {stun_name} message sequence: {sequence}, with error: {err}",
                    );
                    err
                })
        } else {
            Ok(())
        }

    }
}

// #[async_trait::async_trait]
// impl PackageEventTrait<(Ping, Signature)> for Stack {
//     async fn on_package_event(
//         &self,
//         tunnel: DynamicTunnel,
//         head: PackageHeader,
//         head_ext: PackageHeaderExt,
//         data: (Ping, Signature),
//     ) -> NearResult<()> {

//         let requestor = head_ext.requestor();
//         let requestor_object_codec = requestor.object_type_code()?;
//         if let ObjectTypeCode::Device(sub_codec) = requestor_object_codec {
//             if sub_codec == DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8 {
//                 Ok(())
//             } else {
//                 Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, "requestor must core device"))
//             }
//         } else {
//             Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, "requestor is not device"))
//         }?;

//         match &self.0.components.as_ref().unwrap().stun_client.as_ref().unwrap() {
//             CoturnState::S(stun_server) => {
//                 stun_server.on_package_event(tunnel, head, head_ext, data).await
//             }
//             CoturnState::C(stun_client) => {
//                 stun_client.on_package_event(tunnel, head, head_ext, data).await
//             }
//             CoturnState::None => unreachable!("won't reach here"),
//         }
//     }
// }

// #[async_trait::async_trait]
// impl PackageEventTrait<(PingResp, Signature)> for Stack {
//     async fn on_package_event(
//         &self,
//         tunnel: DynamicTunnel,
//         head: PackageHeader,
//         head_ext: PackageHeaderExt,
//         data: (PingResp, Signature),
//     ) -> NearResult<()> {

//         trace!(
//             "Stack::PackageEventTrait::PingResp: tunnel: {}, head: {}, head_ext: {}, PingResp: {}, begin...",
//             tunnel,
//             head, head_ext,
//             data.0
//         );

//         match &self.0.components.as_ref().unwrap().stun_client.as_ref().unwrap() {
//             CoturnState::C(stun_client) => {
//                 let _ = stun_client.on_package_event(tunnel, head, head_ext, data).await;
//             }
//             CoturnState::S(_) => unreachable!("won't reach here."),
//             CoturnState::None => unreachable!("won't reach here."),
//         }

//         Ok(())
//     }
// }

// #[async_trait::async_trait]
// impl PackageEventTrait<(CallReq, Signature)> for Stack {
//     async fn on_package_event(
//         &self,
//         tunnel: DynamicTunnel,
//         head: PackageHeader,
//         head_ext: PackageHeaderExt,
//         data: (CallReq, Signature),
//     ) -> NearResult<()> {

//         trace!(
//             "Stack::PackageEventTrait::CallReq: tunnel: {}, head: {}, head_ext: {}, call_req: {}, begin...",
//             tunnel,
//             head, head_ext,
//             data.0
//         );

//         let requestor = head_ext.requestor();
//         let requestor_type_codec = 
//             requestor.object_type_code()
//                 .map_err(| err | {
//                     error!("requestor's object type code is invalid, with err: {err}");
//                     err
//                 })?;

//         match &self.0.components.as_ref().unwrap().stun_client.as_ref().unwrap() {
//             CoturnState::S(stun_server) => {
//                 match requestor_type_codec {
//                     ObjectTypeCode::Device(codec) if codec == DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8 => { Ok(()) },
//                     _ => {
//                         let error_string = format!("requestor-{requestor_type_codec} has been refuse, {} is not device.", requestor);
//                         warn!("{error_string}");
//                         Err(NearError::new(ErrorCode::NEAR_ERROR_REFUSE, error_string))
//                     }
//                 }?;

//                 stun_server.on_package_event(tunnel, head, head_ext, data).await
//             }
//             CoturnState::C(_) => unreachable!("won't reach here"),
//             CoturnState::None => unreachable!("won't reach here"),
//         }

//     }
// }

// #[async_trait::async_trait]
// impl PackageEventTrait<(CalledReq, Signature)> for Stack {
//     async fn on_package_event(
//         &self,
//         tunnel: DynamicTunnel,
//         head: PackageHeader,
//         head_ext: PackageHeaderExt,
//         data: (CalledReq, Signature),
//     ) -> NearResult<()> {

//         trace!(
//             "Stack::PackageEventTrait::CalledReq: tunnel: {}, head: {}, head_ext: {}, CalledReq: {}, begin...",
//             tunnel,
//             head, head_ext,
//             data.0
//         );

//         let requestor = head_ext.requestor();
//         let requestor_type_codec = 
//             requestor.object_type_code()
//                 .map_err(| err | {
//                     error!("requestor's object type code is invalid, with err: {err}");
//                     err
//                 })?;

//         match &self.0.components.as_ref().unwrap().stun_client.as_ref().unwrap() {
//             CoturnState::C(stun_client) => {
//                 match requestor_type_codec {
//                     ObjectTypeCode::Service(codec) if codec == ServiceObjectSubCode::OBJECT_TYPE_SERVICE_COTURN_MINER as u8 => { Ok(()) },
//                     _ => {
//                         let error_string = format!("requestor-{requestor_type_codec} has been refuse, {} is not sn-service.", requestor);
//                         warn!("{error_string}");
//                         Err(NearError::new(ErrorCode::NEAR_ERROR_REFUSE, error_string))
//                     }
//                 }?;

//                 stun_client.on_package_event(tunnel, head, head_ext, data).await
//             }
//             CoturnState::S(_) => unreachable!("don't reach here."),
//             CoturnState::None => unreachable!("won't reach here"),
//         }

//     }
// }

// #[async_trait::async_trait]
// impl PackageEventTrait<(CalledResp, Signature)> for Stack {
//     async fn on_package_event(
//         &self,
//         tunnel: DynamicTunnel,
//         head: PackageHeader,
//         head_ext: PackageHeaderExt,
//         data: (CalledResp, Signature),
//     ) -> NearResult<()> {

//         trace!(
//             "Stack::PackageEventTrait::CalledResp: tunnel: {}, head: {}, head_ext: {}, CalledResp: {}, begin...",
//             tunnel,
//             head, head_ext,
//             data.0
//         );

//         let requestor = head_ext.requestor();
//         let requestor_type_codec = 
//             requestor.object_type_code()
//                 .map_err(| err | {
//                     error!("requestor's object type code is invalid, with err: {err}");
//                     err
//                 })?;

//         match &self.0.components.as_ref().unwrap().stun_client.as_ref().unwrap() {
//             CoturnState::S(stun_server) => {
//                 match requestor_type_codec {
//                     ObjectTypeCode::Device(codec) if codec == DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8 => { Ok(()) },
//                     _ => {
//                         let error_string = format!("requestor-{requestor_type_codec} has been refuse, {} is not device.", requestor);
//                         warn!("{error_string}");
//                         Err(NearError::new(ErrorCode::NEAR_ERROR_REFUSE, error_string))
//                     }
//                 }?;

//                 stun_server.on_package_event(tunnel, head, head_ext, data).await
//             }
//             _ => unreachable!("won't reach here"),
//         }
//         // unimplemented!()

//     }
// }

// #[async_trait::async_trait]
// impl PackageEventTrait<(CallResp, Signature)> for Stack {
//     async fn on_package_event(
//         &self,
//         tunnel: DynamicTunnel,
//         head: PackageHeader,
//         head_ext: PackageHeaderExt,
//         data: (CallResp, Signature),
//     ) -> NearResult<()> {

//         trace!(
//             "Stack::PackageEventTrait::CallResp: tunnel: {}, head: {}, head_ext: {}, CallResp: {}, begin...",
//             tunnel,
//             head, head_ext,
//             data.0
//         );

//         match &self.0.components.as_ref().unwrap().stun_client.as_ref().unwrap() {
//             CoturnState::C(stun_client) => {
//                 stun_client.on_package_event(tunnel, head, head_ext, data).await
//             }
//             CoturnState::S(_) => unreachable!("won't reach here"),
//             CoturnState::None => unreachable!("won't reach here"),
//         }

//     }
// }

#[async_trait::async_trait]
impl PackageEventTrait<Vec<u8>> for Stack {
    async fn on_package_event(
        &self,
        tunnel: DynamicTunnel,
        head: PackageHeader,
        head_ext: PackageHeaderExt,
        data: Vec<u8>,
    ) -> NearResult<()> {

        let need_transfer = 
            match &self.0.local {
                // when I am box
                StackDevice::CoreService(core) => {
                    if head_ext.to() == core.core_service.object_id() {
                        // target is mine, so I will exec request proc.
                        debug!("target is mine, so I will exec request proc, sequence: {}", head.sequence());
                        false
                    } else {
                        // target isn't me, so I will transfer this request.
                        debug!("target isn't me, so I will transfer this request, sequence: {}", head.sequence());
                        true
                    }
                }
                _ => {
                    // ingore, I will exec requet proc.
                    debug!("ingore, I will exec requet proc., sequence: {}", head.sequence());
                    false
                }
            };

        if !need_transfer {
            let header_meta = HeaderMeta::new(head, head_ext, Some(tunnel.clone_as_interface()))?;

            match &header_meta.command {
                CommandParam::Request(_) => self.on_request_process(tunnel, header_meta, data).await,
                CommandParam::Response(_) => self.on_response_process(tunnel, header_meta, data).await,
            }
        } else {
            self.on_transfer_process(tunnel, head, head_ext, data).await
            // let (major, sequence) = head.split();
            // let (source, target, topic) = head_ext.split();

            // PostMessageTrait::post_message(
            //         self,
            //         (
            //         RequestorMeta {
            //             sequence: Some(sequence),
            //             creator: Some(CreatorMeta{
            //                 creator: source.creator, 
            //                 creator_local: source.creator_local,
            //                 creator_remote: source.creator_remote,
            //             }),
            //             requestor: Some(self.local_device_id().clone()), 
            //             to: Some(target), 
            //             topic: topic.map(| topic | topic.into()),
            //             need_sign: false,
            //             ..Default::default()
            //         }, 
            //         {
            //             match major {
            //                 MajorCommand::Request => AnyNamedRequest::with_request(data.into()),
            //                 MajorCommand::Response => AnyNamedRequest::with_response(data.into()),
            //                 _ => unreachable!()
            //             }
            //         },
            //         None
            //         )
            //     )
            //     .await
            //     .map(| _ | ())

        }
    }
}

#[async_trait::async_trait]
impl PackageFailureTrait for Stack {

    async fn on_package_failure(
        &self, 
        error: NearError,
        data: DataContext,
    ) {
        let sequence = data.head.sequence();
        let target = data.head_ext.to();
        let major_command = data.head.major_command();

        trace!("Stack::on_package_failure, sequence: {sequence}, major_command: {major_command}, target: {target}, error: {error}");

        let device_type_codec = 
            if let Ok(device_type_codec) = self.local_device_id().object_type_code() {
                device_type_codec
            } else {
                warn!("[{sequence}] get device codec exception.");
                return;
            };

        match major_command {
            MajorCommand::Stun => {
                match &self.0.components.as_ref().unwrap().coturn_state.as_ref().unwrap() {
                    CoturnState::C(coturn_client) => {
                        if let ObjectTypeCode::Device(_) = device_type_codec {
                            coturn_client.stun_client.on_package_failure(error, data).await;
                        } else {
                            unreachable!("invalid device_type_codec");
                        }
                    }
                    CoturnState::S(coturn_server) => {
                        if let ObjectTypeCode::Service(_) = device_type_codec {
                            coturn_server.stun_server.on_package_failure(error, data).await;
                        } else {
                            unreachable!("invalid device_type_codec");
                        }
                    }
                    CoturnState::None => unreachable!("won't reach here"),
                }
            }
            MajorCommand::Request | MajorCommand::Response => {
                
            }
            MajorCommand::Exchange | MajorCommand::AckTunnel | MajorCommand::AckAckTunnel | MajorCommand::Ack | MajorCommand::AckAck => return,
            MajorCommand::None => unreachable!("don't reach here")
        }

    }

}

#[async_trait::async_trait]
impl PackageEstablishedTrait for Stack {
    async fn on_established(&self, tunnel: DynamicTunnel) {
        trace!("Stack::on_established, tunnel: {tunnel}");

        // let tunnel_type_codec = 
        if let Ok(remote_object_codec) = tunnel.peer_id().object_type_code() {
            match remote_object_codec {
                ObjectTypeCode::Service(codec) if codec == ServiceObjectSubCode::OBJECT_TYPE_SERVICE_COTURN_MINER as u8 => {
                }
                _ => { /* ignore */ }
            }
        }

        // check device codec
        let stun_client = {
            match self.local_device_id().object_type_code() {
                Ok(code) => {
                    if let ObjectTypeCode::Device(_) = code {
                        match &self.0.components.as_ref().unwrap().coturn_state.as_ref().unwrap() {
                            CoturnState::C(stun_client) => Some(stun_client),
                            CoturnState::S(_) => None,
                            CoturnState::None => unreachable!("won't reach here"),
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        };

        if let Some(coturn_client) = stun_client {
            coturn_client.stun_client.on_established(tunnel).await
        }
    }
}

impl Stack {

    pub(self) async fn on_transfer_process(
        &self,
        tunnel: DynamicTunnel,
        head: PackageHeader,
        head_ext: PackageHeaderExt,
        data: Vec<u8>,
    ) -> NearResult<()> {

        trace!(
            "request: tunnel: {}, head: {}, head_ext: {} begin...",
            tunnel,
            head, head_ext
        );

        let (major, sequence) = head.split();
        let (source, target, topic) = head_ext.split();

        struct TransferCallbackRoutine {
            requestor: ObjectId,
            topic: Option<Topic>,
        }

        #[async_trait::async_trait]
        impl RoutineEventTrait for TransferCallbackRoutine {
            async fn emit(&self, header_meta: &HeaderMeta, data: Vec<u8>) -> NearResult<EventTextResult> {

                trace!(
                    "TransferCallbackRoutine::emit, target: {} sequence: {}, data-size: {}", 
                    &self.requestor, 
                    header_meta.sequence(), 
                    data.len()
                );

                Ok(EventTextResult::Transfer(TransferEvent{
                    to: vec![(self.requestor.clone(), None)],
                    topic: {
                        self.topic.as_ref().map(| topic | topic.clone().into())
                            .unwrap_or(String::new())
                    },
                    data,
                }))

            }
        }

        let cb = 
            TransferCallbackRoutine {
                requestor: source.requestor().clone(),
                topic: topic.as_ref().map(| topic | topic.clone().into()),
            };

        let this = self.clone();
        let _ = 
            PostMessageTrait::post_message(
                    &this,
                    (
                        RequestorMeta {
                            sequence: Some(sequence),
                            creator: Some(CreatorMeta{
                                creator: source.creator, 
                                creator_local: source.creator_local,
                                creator_remote: source.creator_remote,
                            }),
                            requestor: Some(self.local_device_id().clone()), 
                            to: Some(target), 
                            topic: topic.map(| topic | topic.into()),
                            need_sign: false,
                            ..Default::default()
                        }, 
                        {
                            match major {
                                MajorCommand::Request => AnyNamedRequest::with_request(data.into()),
                                MajorCommand::Response => AnyNamedRequest::with_response(data.into()),
                                _ => unreachable!()
                            }
                        },
                        Some(Box::new(cb) as Box<dyn RoutineEventTrait>),
                    )
                )
                .await?;

        Ok(())
    }

    pub(self) async fn on_request_process(
        &self,
        tunnel: DynamicTunnel,
        header_meta: HeaderMeta,
        data: Vec<u8>,
    ) -> NearResult<()> {
        trace!(
            "request: tunnel: {}, head: {}, begin...",
            tunnel,
            header_meta
        );

        let creator = &header_meta.creator;
        let sender = &header_meta.requestor;
        let _to = &header_meta.to;
        let topic = &header_meta.topic;
        let topic_ref = topic.topic_d()?;
        let sequence = header_meta.sequence();

        let routine = 
            self.process_impl()
                .create_routine(sender, &topic_ref)
                .map_err(|err| {
                    warn!("failed to create_routine({topic_ref}) with err {err} and sequence: {sequence}");
                    err
                })?;

        let r =
            routine.emit(&header_meta, data)
                .await
                .map_err(| err | {
                    error!("failed to on_route() with err {}, at creator: {:?}, sender: {}, topic: {}, sequest: {}",
                        err, creator, sender, topic_ref, sequence);
                    err
                })?;

        match r {
            EventTextResult::Response(data) => {

                PostMessageTrait::post_message(
                    self, 
                    (
                        Some(tunnel.clone()),
                        RequestorMeta {
                            sequence: Some(sequence.clone()),
                            creator: creator.clone(),
                            requestor: Some(self.local_device_id().clone()),
                            to: Some(tunnel.peer_id().clone()),
                            topic: Some(topic.clone().into()),
                            ..Default::default()
                        },
                        AnyNamedRequest::with_response(data.data.into()),
                        None,
                    )
                )
                .await
                .map(| _ | ())

            }
            EventTextResult::Transfer(data) => {
                let (mut to, topic, data) = data.split();
                let mut futs = vec![];
                let last = to.pop();

                for (target, callback) in to {
                    futs.push(
                        PostMessageTrait::post_message(
                            self,
                            (
                            RequestorMeta {
                                sequence: Some(sequence.clone()),
                                creator: creator.clone(),
                                requestor: Some(self.local_device_id().clone()),
                                to: Some(target),
                                topic: Some(topic.clone().into()),
                                ..Default::default()
                            }, 
                            {
                                match &header_meta.command {
                                    CommandParam::Request(_) => AnyNamedRequest::with_request(data.clone().into()),
                                    CommandParam::Response(_) => AnyNamedRequest::with_response(data.clone().into()),
                                }
                            },
                            callback
                            )
                        )
                    );
                }

                if let Some((target, callback)) = last {
                    futs.push(
                        PostMessageTrait::post_message(
                            self,
                            (
                            RequestorMeta {
                                sequence: Some(sequence.clone()),
                                creator: creator.clone(),
                                requestor: Some(self.local_device_id().clone()),
                                to: Some(target),
                                topic: Some(topic.into()),
                                ..Default::default()
                            }, 
                            {
                                match &header_meta.command {
                                    CommandParam::Request(_) => AnyNamedRequest::with_request(data.clone().into()),
                                    CommandParam::Response(_) => AnyNamedRequest::with_response(data.clone().into()),
                                }
                            },
                            callback
                            )
                        )
                    );
                }

                let _ = futures::future::join_all(futs).await;

                Ok(())
            }
            _ => {
                /* Ignore */
                Ok(())
            }
        }
    }

    pub(self) async fn on_response_process(
        &self,
        tunnel: DynamicTunnel,
        head_ext: HeaderMeta,
        data: Vec<u8>,
    ) -> NearResult<()> {
        trace!("response: tunnel: {}, head: {}, begin...", tunnel, head_ext);

        let creator = &head_ext.creator;
        let topic = &head_ext.topic;
        let _topic_ref = topic.topic_d()?;
        let sender = &head_ext.requestor;
        let sequence = head_ext.command.sequence();

        let routine = 
            match self.event_manager().take_routine(sender, sequence, 0) {
                Some((_, routine)) => Ok(routine),
                None => {
                    let error_string = format!(
                        "not found routine, sender: {}, topic: {}, sequence: {}",
                        sender, topic, sequence
                    );
                    warn!("{error_string}");
                    Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string))
                }
            }?;

        let r = 
            routine.emit(&head_ext, data).await.map_err(|err| {
                error!(
                    "failed to on_route() with err {}, at sender: {}, topic: {}, sequence: {}",
                    err, sender, topic, sequence
                );
                err
            })?;

        match r {
            EventTextResult::Response(data) => {

                PostMessageTrait::post_message(
                    self, 
                    (
                        Some(tunnel.clone()),
                        RequestorMeta {
                            sequence: Some(sequence.clone()),
                            creator: creator.clone(),
                            requestor: Some(self.local_device_id().clone()),
                            to: Some(tunnel.peer_id().clone()),
                            topic: Some(topic.clone().into()),
                            ..Default::default()
                        },
                        AnyNamedRequest::with_response(data.data.into()),
                        None,
                    )
                )
                .await
                .map(| _ | ())

            }
            EventTextResult::Transfer(data) => {
                let (mut to, topic, data) = data.split();
                let mut futs = vec![];
                let last = to.pop();

                for (target, callback) in to {
                    futs.push(
                        PostMessageTrait::post_message(
                            self,
                            (
                            RequestorMeta {
                                sequence: Some(sequence.clone()),
                                creator: creator.clone(),
                                requestor: Some(self.local_device_id().clone()),
                                to: Some(target),
                                topic: Some(topic.clone().into()),
                                ..Default::default()
                            }, 
                            {
                                match &head_ext.command {
                                    CommandParam::Request(_) => AnyNamedRequest::with_request(data.clone().into()),
                                    CommandParam::Response(_) => AnyNamedRequest::with_response(data.clone().into()),
                                }
                            },
                            callback
                            )
                        )
                    );
                }

                if let Some((target, callback)) = last {
                    futs.push(
                        PostMessageTrait::post_message(
                            self,
                            (
                            RequestorMeta {
                                sequence: Some(sequence.clone()),
                                creator: creator.clone(),
                                requestor: Some(self.local_device_id().clone()),
                                to: Some(target),
                                topic: Some(topic.into()),
                                ..Default::default()
                            }, 
                            {
                                match &head_ext.command {
                                    CommandParam::Request(_) => AnyNamedRequest::with_request(data.clone().into()),
                                    CommandParam::Response(_) => AnyNamedRequest::with_response(data.clone().into()),
                                }
                            },
                            callback
                            )
                        )
                    );
                }

                let _ = futures::future::join_all(futs).await;

                Ok(())
            }
            _ => {
                /* Ignore */
                Ok(())
            }
        }
    }
}

#[async_trait::async_trait]
impl TunnelEventTrait for Stack {
    async fn on_reconnect(&self, ep: Endpoint, target: &ObjectId) -> NearResult<()> {
        self.tunnel_event().on_reconnect(ep, target).await?;

        match &self.0.local {
            StackDevice::Runtime(_) => {
                // 
                self.0.events.process_event_impl.on_reinit();
            },
            StackDevice::People(_) => {
                self.0.events.process_event_impl.on_reinit();
            }
            StackDevice::CoreService(_) | StackDevice::CoturnMiner(_) => { /* ignore */ }
        }

        Ok(())
        // match self.0
    }
}

#[async_trait::async_trait]
impl SignerTrait for Stack {
    fn public_key(&self) -> &PublicKey {
        match &self.0.local {
            StackDevice::CoreService(param) | 
            StackDevice::CoturnMiner(param) => param.core_service.desc().public_key().as_ref().unwrap(),
            StackDevice::Runtime(_) => {
                unreachable!()
            }
            StackDevice::People(param) => param.people.desc().public_key().as_ref().unwrap(),
        }
    }

    async fn sign(&self, data: &[u8]) -> NearResult<Signature> {
        match &self.0.local {
            StackDevice::CoreService(param) | 
            StackDevice::CoturnMiner(param) => param.core_service_private_key.sign(data),
            StackDevice::Runtime(_) => {
                unreachable!()
            }
            StackDevice::People(param) => param.people_private_key.sign(data),
        }
    }
}
