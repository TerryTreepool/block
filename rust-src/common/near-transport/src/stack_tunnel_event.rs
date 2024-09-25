
use log::{error, info, trace, warn, };
use near_base::{DeviceObjectSubCode, Endpoint, ErrorCode, NearError, NearResult, ObjectId, ObjectTypeCode};

use crate::{Stack, tunnel::TunnelEventTrait};

pub struct RuntimeTunnelEvent {
    stack: Stack,
}

impl RuntimeTunnelEvent {
    pub fn new(stack: Stack) -> Self {
        Self{
            stack,
        }
    }
}

#[async_trait::async_trait]
impl TunnelEventTrait for RuntimeTunnelEvent {
    async fn on_reconnect(&self, ep: Endpoint, target: &ObjectId) -> NearResult<()> {
        trace!("TunnelEventTrait::on_closed, ep={ep}, target:{target}");

        match target.object_type_code() {
            Ok(code) => {
                match code {
                    ObjectTypeCode::Device(v) if v == DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8 => { Ok(()) },
                    ObjectTypeCode::Device(v) => {
                        let error_string = format!("can't reconnect {target} because target type codec unkown, expor: {v}");
                        error!("{error_string}");
                        Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, error_string))
                    }
                    _ => {
                        let error_string = format!("{} target object type code is not device.", target);
                        error!("{error_string}");
                        Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, error_string))
                    }
                }
            }
            Err(err) => {
                error!("{} target object type code is invalid with err: {}", target, err);
                Err(err)
            }
        }?;

        let target_device = 
            self.stack
                .cacher_manager()
                .get(target).await
                .ok_or_else(|| {
                let error_string = format!("not found {target} target.");
                warn!("{error_string}");
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
            })?;

        if target_device.object_id() != target {
            let error_string = format!("Target and CoreService ID are not equal, except={} got={}", target, self.stack.core_device().object_id());
            error!("{error_string}");
            return Err(NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, error_string));
        }

        if !target_device.body().content().endpoints().contains(&ep) {
            let error_string = format!("Not found {} in target endpoints.", ep);
            error!("{error_string}");
            return Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string));
        }

        if ep.is_tcp() {
            match async_std::future::timeout(
                    self.stack.config().tunnel.container.connect_timeout, 
                    self.stack
                        .net_manager()
                        .connect_tcp_interface(&ep, self.stack.core_device())
                ).await {

                Ok(r) => {
                    if let Err(e) = r {
                        error!("failed connect {target} with err = {e}");
                        Err(e)
                    } else {
                        info!("successfule connect {target}");
                        Ok(())
                    }
                }
                Err(e) => {
                    error!("connect {target} timeout.");
                    Err(NearError::new(ErrorCode::NEAR_ERROR_TIMEOUT, format!("timeout: e={e}")))
                }

            }
        } else if ep.is_udp() {
            todo!()
        } else {
            unreachable!("don't reach here.")
        }
    }
}

pub struct ServiceTunnelEvent {
    #[allow(unused)]
    stack: Stack,
}

impl ServiceTunnelEvent {
    pub fn new(stack: Stack) -> Self {
        Self{
            stack,
        }
    }
}

#[async_trait::async_trait]
impl TunnelEventTrait for ServiceTunnelEvent {
    async fn on_reconnect(&self, ep: Endpoint, target: &ObjectId) -> NearResult<()> {
        info!("remote has been disconnected, ep: {}, target: {}", ep, target);

        let target_object_codec = 
            target.object_type_code()
                .map_err(| err | {
                    error!("{} target object type code is invalid with err: {}", target, err);
                    err
                })?;

        match target_object_codec {
            ObjectTypeCode::Device(_) => {
                let error_string = format!("{} target object type code is device. you must use ping::call function with peer", target);
                warn!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_IGNORE, error_string))
            }
            ObjectTypeCode::Service(_)=> { Ok(()) },
            _ => {
                let error_string = format!("{} target object type code is not device or service.", target);
                error!("{error_string}");
                Err(NearError::new(ErrorCode::NEAR_ERROR_IGNORE, error_string))
            }
        }?;

        let target_device = 
            self.stack
                .cacher_manager()
                .get(target).await
                .ok_or_else(|| {
                    let error_string = format!("not found {target} target.");
                    warn!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
                })?;

        if target_device.object_id() != target {
            let error_string = format!("Target and CoreService ID are not equal, except={} got={}", target, self.stack.core_device().object_id());
            error!("{error_string}");
            return Err(NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, error_string));
        }

        if !target_device.body().content().endpoints().contains(&ep) {
            let error_string = format!("Not found {} in target endpoints.", ep);
            error!("{error_string}");
            return Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string));
        }

        match target_object_codec {
            ObjectTypeCode::Service(_)=> {
                let _ = self.stack.stun_client().reset_sn(target_device, Some(ep)).await?;
            }
            _ => {
                unreachable!("don't reach here.")
            }            
        }

        Ok(())
    }
}
