
use std::{collections::{btree_map::Entry, BTreeMap}, ops::Deref, sync::{Arc, RwLock}, time::Duration };
use log::{error, trace};

use near_base::{sequence::SequenceString, *};

use crate::{h::OnTimeTrait, network::{DataContext, TcpInterface, TcpPackageEventTrait, UdpInterface, UdpPackageEventTrait 
            }, package::{MajorCommand, PackageDataSet }, Stack };
use super::{container::{TunnelContainer, TunnelGuard}, tunnel::State, DynamicTunnel, PostMessageTrait };
use super::tcp::Tunnel as TcpTunnel;
use super::udp::Tunnel as UdpTunnel;

#[derive(Clone)]
pub struct Config {
    pub max_task_count: usize,
    pub package_wait_interval: Duration,
    // pub connect_timeout: Duration,
    // pub confirm_timeout: Duration,
    // pub accept_timeout: Duration,
    // // 调用retain_keeper 之后延迟多久开始尝试从preactive 进入 active状态
    // pub retain_connect_delay: Duration,
    // pub ping_interval: Duration,
    // pub ping_timeout: Duration,

    // pub package_buffer: usize,
    // pub piece_buffer: usize,
    // // 检查发送piece buffer的间隔
    // pub piece_interval: Duration,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            max_task_count: 4,
            package_wait_interval: Duration::from_secs(1),
        }
    }
}

// #[derive(PartialEq, Eq, PartialOrd, Ord)]
// struct MessageTag {
//     sequence: SequenceString,
//     timestamp: Timestamp
// }

// type MessageRef = Arc<Message>;

// struct ResendQueue {
//     container: TunnelContainer,
// }

struct ManagerImpl {
    stack: Stack,
    entries: RwLock<BTreeMap<ObjectId, TunnelGuard>>,
    resender_queue: RwLock<BTreeMap<ObjectId, TunnelContainer>>,
    recyle_queue: RwLock<BTreeMap<ObjectId, TunnelContainer>>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub(crate) fn open(stack: Stack) -> NearResult<Self> {
        let manager =
            Self(Arc::new(ManagerImpl{
                stack,
                entries: RwLock::new(BTreeMap::new()),
                resender_queue: RwLock::new(Default::default()),
                recyle_queue: RwLock::new(Default::default()),
            }));

        Ok(manager)
    }

    #[inline]
    pub(super) fn as_stack(&self) -> &Stack {
        &self.0.stack
    }

    pub fn container_of(&self, id: &ObjectId) -> Option<TunnelGuard> {
        self.0.entries.read().unwrap()
            .get(&id)
            .map(|tunnel| {
                tunnel.clone()
            })
    }

    pub fn create_container(&self, remote: &ObjectId) -> TunnelGuard {
        let entries = &mut *self.0.entries.write().unwrap();

        if let Some(tunnel) = entries.get(remote) {
            tunnel.clone()
        } else {
            let tunnel = TunnelGuard::new(TunnelContainer::new(self.clone(), remote.clone()));
            entries.insert(remote.clone(), tunnel.clone());
            tunnel
        }
    }

    #[allow(unused)]
    pub async fn wait_tunnel_active(&self, remote: &ObjectId) -> State {
        if let Some(tunnel_container) = self.container_of(remote) {
            tunnel_container.wait_active().await
        } else {
            State::Dead
        }
    }

    #[allow(unused)]
    pub fn test_tunnel_active(&self, remote: &ObjectId) -> State {
        if let Some(tunnel_container) = self.container_of(remote) {
            tunnel_container.to_state()
        } else {
            State::Dead
        }
    }

    #[allow(unused)]
    pub fn close_tunnel(&self, tunnel: DynamicTunnel) {
        if let Some(guard) = self.container_of(tunnel.peer_id()) {
            guard.close_tunnel(tunnel);
        }
    }
}

// resender queue 
impl Manager {
    pub(super) fn append_resender(&self, tunnel: TunnelContainer) {
        let mut_queue = &mut *self.0.resender_queue.write().unwrap();

        match mut_queue.entry(tunnel.remote_id().clone()) {
            Entry::Vacant(empty) => {
                empty.insert(tunnel);
            }
            _ => {}
        }
    }

    pub(super) fn remove_resender(&self, remote_id: &ObjectId) {
        let _ = 
            self.0
                .resender_queue
                .write().unwrap()
                .remove(remote_id);
    }

    pub(self) fn on_time_escape_for_resend(&self, now: Timestamp) {

        let queue: Vec<TunnelContainer> = {
            self.0.resender_queue
                .read().unwrap()
                .values()
                .cloned()
                .collect()
        };

        for q in queue.iter() {
            q.on_time_escape_for_resend(now);
        }
    }

}

// recyle queue
impl Manager {
    pub(super) fn append_recyle(&self, tunnel: TunnelContainer) {
        let mut_queue = &mut *self.0.recyle_queue.write().unwrap();

        match mut_queue.entry(tunnel.remote_id().clone()) {
            Entry::Vacant(empty) => {
                empty.insert(tunnel);
            }
            _ => {}
        }
    }

    pub(super) fn remove_recyle(&self, remote_id: &ObjectId) {
        let _ = 
            self.0
                .recyle_queue
                .write().unwrap()
                .remove(remote_id);
    }

    pub(self) fn on_time_escape_for_recyle(&self, now: Timestamp) {
        let queue: Vec<TunnelContainer> = {
            self.0.recyle_queue
                .read().unwrap()
                .values()
                .cloned()
                .collect()
        };

        for q in queue.iter() {
            q.on_time_escape_for_recyle(now);
        }
    }
}

// #[async_trait::async_trait]
// impl PostMessageTrait<(HeaderMeta, Data)> for Manager {
//     async fn post_message(&self, context: (HeaderMeta, Data)) -> NearResult<()> {

//         let (header_meta, body) = context;

//         let stack = self.as_stack();

//         let target = if stack.is_core() {
//             &header_meta.to
//         } else {
//             stack.core_device().object_id()
//         };
//         let target_type_codec = 
//             target.object_type_code().map_err(| err | {
//                 error!("invalid target type codec with {}", err);
//                 err
//             })?;

//         trace!("post_message: target: {}, requestor: {}, sequence: {}", 
//             target, 
//             header_meta.requestor,
//             header_meta.sequence()
//         );

//         let tunnel = match self.container_of(target) {

//             Some(tunnel) => { Ok(tunnel) }
//             None => {
//                 // self.stack().on_lack(target).await;

//                 let error_message = format!("Not found {}-{} tunnel.", target_type_codec, target);
//                 error!("failed send package with {}", error_message);
//                 Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_message))
//             }
//         }?;

//         tunnel.post_message((header_meta, body))
//             .await
//             .map_err(| err | {
//                 error!("{}", err);
//                 err
//             })
//     }
// }

// #[async_trait::async_trait]
// impl PostMessageTrait<(DynamicTunnel, HeaderMeta, Data)> for Manager {
//     async fn post_message(&self, context: (DynamicTunnel, HeaderMeta, Data)) -> NearResult<()> {
//         let (target, header_meta, data) = context;

//         if target.local().is_tcp() {
//             target.clone_as_tunnel::<TcpTunnel>()
//                 .post_message((
//                     HeaderMeta {
//                         command: header_meta.command,
//                         /// 创建者
//                         creator: header_meta.creator,
//                         /// 请求者
//                         requestor: header_meta.requestor,
//                         /// 目的地
//                         to: target.peer_id().clone(),
//                         /// 主题
//                         topic: header_meta.topic,
//                         /// Net元素
//                         net_meta: header_meta.net_meta,
//                     }, 
//                     data
//                 ))
//                 .await
//         } else if target.local().is_tcp() {
//             target.clone_as_tunnel::<UdpTunnel>()
//                 .post_message((
//                     HeaderMeta {
//                         command: header_meta.command,
//                         /// 创建者
//                         creator: header_meta.creator,
//                         /// 请求者
//                         requestor: header_meta.requestor,
//                         /// 目的地
//                         to: target.peer_id().clone(),
//                         /// 主题
//                         topic: header_meta.topic,
//                         /// Net元素
//                         net_meta: header_meta.net_meta,
//                     }, 
//                     data
//                 ))
//                 .await
//         } else {
//             unreachable!()
//         }
//     }
// }

#[async_trait::async_trait]
impl PostMessageTrait<(DynamicTunnel, SequenceString, PackageDataSet)> for Manager {

    type R = ();

    async fn post_message(
        &self, 
        context: (DynamicTunnel, SequenceString, PackageDataSet)
    ) -> NearResult<Self::R> {
        let (tunnel, sequence, package) = context;

        if tunnel.local().is_tcp() {
            tunnel.clone_as_tunnel::<TcpTunnel>()
                .post_message((sequence, package))
                .await
        } else if tunnel.local().is_udp() {
            tunnel.clone_as_tunnel::<UdpTunnel>()
                .post_message((sequence, package))
                .await
        } else {
            unreachable!("don't reach here.")
        }
    }
}

#[async_trait::async_trait]
impl PostMessageTrait<(ObjectId, SequenceString, PackageDataSet)> for Manager {

    type R = ();

    async fn post_message(
        &self, 
        context: (ObjectId, SequenceString, PackageDataSet)
    ) -> NearResult<Self::R> {
        let (target, sequence, package) = context;

        trace!("post_message: target: {}, sequence: {}", 
            target, sequence
        );

        let tunnel = match self.container_of(&target) {

            Some(tunnel) => { Ok(tunnel) }
            None => {
                let error_message = format!("Not found {} tunnel.", target);
                error!("failed send package with {}", error_message);
                Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_message))
            }
        }?;

        tunnel.post_message((sequence, package)).await
    }
}

impl Manager {
    // async fn run_loop(&self) {
    //     let stack = self.stack();
    //     let quque_guard = self.0.queue.clone();
    //     let manager_config = &stack.config().tunnel.manager;

    //     loop {
    //         // if let Some((target, package_data)) =
    //         //     quque_guard.wait_and_take(manager_config.package_wait_interval).await {

    //         //     // TODO: add exit flag in future

    //         //     let arc_self = self.clone();
    //         //     async_std::task::spawn(async move {
    //         //         let _ = arc_self.send_multi_package(target, package_data.clone()).await;
    //         //     });
    //         // }
    //     }
    // }
        // if let Err(err) = tunnel.post_message_with_builder(topic, data, builder, cb).await {
        //     error!("{}", err);
        //     match err.errno() {
        //         ErrorCode::NEAR_ERROR_NO_AVAILABLE | ErrorCode::NEAR_ERROR_UNACTIVED | ErrorCode::NEAR_ERROR_TIMEOUT => {
        //             self.stack().on_closed(target).await
        //         }
        //         _ => {
        //         },
        //     }

        //     Err(err)
        // } else {
        //     Ok(())
        // }
    // pub(crate) async fn post_message(
    //     &self,
    //     header_meta: HeaderMeta,
    //     body: Data,
    // ) -> NearResult<()> {

    //     let stack = self.as_stack();

    //     let target = if stack.is_core() {
    //         &header_meta.to
    //     } else {
    //         stack.core_device().object_id()
    //     };

    //     trace!("post_message: target: {}, requestor: {}, sequence: {}", 
    //         target, 
    //         header_meta.requestor,
    //         header_meta.sequence()
    //     );

    //     let tunnel = match self.container_of(target) {

    //         Some(tunnel) => { Ok(tunnel) }
    //         None => {
    //             // self.stack().on_lack(target).await;

    //             let error_message = format!("Not found {} tunnel.", target);
    //             error!("failed send package with {}", error_message);
    //             Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_message))
    //         }
    //     }?;

    //     tunnel.post_message(header_meta, body)
    //         .await
    //         .map_err(| err | {
    //             error!("{}", err);
    //             err
    //         })
    // }

    // pub(crate) async fn post_message_with_tunnel(&self,
    //                                              target: DynamicTunnel,
    //                                              header_meta: HeaderMeta,
    //                                              data: Data) -> NearResult<()> {
    //     if target.local().is_tcp() {
    //         target.clone_as_tunnel::<TcpTunnel>()
    //             .post_data(
    //                 HeaderMeta {
    //                     command: header_meta.command,
    //                     /// 创建者
    //                     creator: header_meta.creator,
    //                     /// 请求者
    //                     requestor: header_meta.requestor,
    //                     /// 目的地
    //                     to: target.peer_id().clone(),
    //                     /// 主题
    //                     topic: header_meta.topic,
    //                     /// Net元素
    //                     net_meta: header_meta.net_meta,
    //                 }, 
    //                 data
    //             )
    //             .await
    //         // target.clone_as_tunnel::<TcpTunnel>().post_data(header_meta, data).await
    //     } else if target.local().is_tcp() {
    //         target.clone_as_tunnel::<UdpTunnel>()
    //             .post_data(
    //                 HeaderMeta {
    //                     command: header_meta.command,
    //                     /// 创建者
    //                     creator: header_meta.creator,
    //                     /// 请求者
    //                     requestor: header_meta.requestor,
    //                     /// 目的地
    //                     to: target.peer_id().clone(),
    //                     /// 主题
    //                     topic: header_meta.topic,
    //                     /// Net元素
    //                     net_meta: header_meta.net_meta,
    //                 }, 
    //                 data
    //             )
    //             .await
    //         // target.clone_as_tunnel::<UdpTunnel>().post_data(header_meta, data).await
    //     } else {
    //         unreachable!()
    //     }
    // }


    // async fn send_multi_package(&self, target: Vec<ObjectGuard>, package_data: PackageDataSet) -> NearResult<()> {
    //     struct Sender {
    //         target: Vec<ObjectGuard>,
    //         package: PackageDataSet,
    //     }

    //     impl std::iter::Iterator for Sender {
    //         type Item = (ObjectGuard, PackageDataSet);
    //         fn next(&mut self) -> Option<Self::Item> {
    //             if let Some(target) = self.target.pop() {
    //                 Some((target, self.package.clone()))
    //             } else {
    //                 None
    //             }
    //         }
    //     }

    //     let sender = Sender {
    //         target,
    //         package: package_data,
    //     };

    //     for (target, package) in sender {
    //         let _ = self.send_package(target, package).await;
    //     }

    //     Ok(())
    // }

    // async fn send_package(&self, target: ObjectGuard, package: PackageDataSet) -> NearResult<usize> {
    //     let tunnel =
    //         self.container_of(target.object_id())
    //             .ok_or_else(|| {
    //                 let error_message = format!("Not found {} tunnel.", target);
    //                 error!("failed send package with {}", error_message);
    //                 NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_message)
    //             })?;

    //     let package_count = package.dataset_count();
    //     let mut sended_finished = 0;
    //     for index in 0..package_count {
    //         let package_data = package.dataset(index).unwrap();
    //         match tunnel.send_raw_data(package_data.as_slice()) {
    //             Ok(len) => { sended_finished = sended_finished + len; }
    //             Err(err) => {
    //                 let error_message = format!("fail send to target: {}, with err: {}", target, err);
    //                 error!("failed send package with {}", error_message);
    //                 return Err(err);
    //             }
    //         }
    //     }

    //     info!("successful send to target: {}, seq: {}, len: {}", target, package.package_head().sequence().into_value(), sended_finished);

    //     Ok(sended_finished)
    // }

}

#[async_trait::async_trait]
impl TcpPackageEventTrait for Manager {
    fn on_connected(&self, interface: TcpInterface, remote: &DeviceObject) {
        let tunnel = self.create_container(remote.object_id());

        TcpPackageEventTrait::on_connected(tunnel.deref(), interface, remote);
        // if self.container_of(remote.object_id()).is_some() {
        //     return;
        // }

        // match self.create_container(&ObjectGuard::from(remote.clone())) {
        //     Ok(tunnel) => {
        //         tunnel.on_connected(interface, remote);
        //     }
        //     Err(err) => {
        //         error!("failed create tunnel with err: {}", err);
        //     }
        // }
    }

    fn on_closed(&self, interface: &TcpInterface, remote: &ObjectId) {
        trace!("on_closed: remote:{remote}, interface: {interface}");

        if let Some(tunnel) = self.container_of(remote) {
            tunnel.on_closed(interface, remote)
        }
    }

    async fn on_tcp_package(
        &self, 
        interface: TcpInterface, 
        data_context: DataContext
    ) -> NearResult<()> {
        trace!("on_tcp_package: head={}, head-ext={}", data_context.head, data_context.head_ext);

        let tunnel = {
            if let Some(tunnel) = self.container_of(data_context.head_ext.requestor()) {
                tunnel
            } else {
                let tunnel = {
                    match data_context.head.major_command() {
                        MajorCommand::Exchange => {
                            Ok(self.create_container(data_context.head_ext.requestor()))
                        }
                        _ => {
                            let error_string = format!("{} must exchange first", data_context.head_ext.requestor());
                            error!("{error_string}");
                            Err(NearError::new(ErrorCode::NEAR_ERROR_PROTOCOL_NEED_EXCHANGE, error_string))
                        }
                    }
                }?;

                tunnel
            }
        };

        tunnel.on_tcp_package(interface, data_context).await
    }

}

#[async_trait::async_trait]
impl UdpPackageEventTrait<Endpoint> for Manager {

    fn on_connected(&self, interface: UdpInterface, remote: &DeviceObject, remote_endpoint: Endpoint) {
        let tunnel = self.create_container(remote.object_id());

        UdpPackageEventTrait::on_connected(tunnel.deref(), interface, remote, remote_endpoint);
    }

    async fn on_udp_package(
        &self, 
        interface: UdpInterface, 
        data_context: DataContext, 
        remote: Endpoint
    ) -> NearResult<()> {
        trace!("on_udp_package: head={}, head-ext={}", data_context.head, data_context.head_ext);

        let tunnel = {
            if let Some(tunnel) = self.container_of(data_context.head_ext.requestor()) {
                tunnel
            } else {
                let tunnel = {
                    match data_context.head.major_command() {
                        MajorCommand::Exchange => {
                            Ok(self.create_container(data_context.head_ext.requestor()))
                        }
                        _ => {
                            let error_string = format!("{} must exchange first", data_context.head_ext.requestor());
                            error!("{error_string}");
                            Err(NearError::new(ErrorCode::NEAR_ERROR_PROTOCOL_NEED_EXCHANGE, error_string))
                        }
                    }
                }?;

                tunnel
            }
        };

        tunnel.on_udp_package(interface, data_context, remote).await

        // let tunnel = {
        //     if let Some(tunnel) = self.container_of(package..requestor()) {
        //         tunnel
        //     } else {
        //         let tunnel = {
        //             match package.as_head().major_command() {
        //                 MajorCommand::Exchange => {
        //                     let exchange: &Exchange = package.as_ref();
        //                     self.create_container(&exchange.from_device)
        //                         .map(|tunnel| tunnel.clone())
        //                         .map_err(|err| err)
        //                 }
        //                 _ => {
        //                     Err(NearError::new(ErrorCode::NEAR_ERROR_PROTOCOL_NEED_EXCHANGE,
        //                                     format!("{} must exchange first", package.as_headext().requestor())))
        //                 }
        //             }
        //         }?;

        //         tunnel
        //     }
        // };

        // tunnel.on_udp_package(interface, package, remote)
    }


}

impl OnTimeTrait for Manager {
    fn on_time_escape(&self, now: Timestamp) {
        self.on_time_escape_for_recyle(now);
        self.on_time_escape_for_resend(now);
    }
}
