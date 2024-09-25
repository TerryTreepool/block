
use near_base::{DeviceObject, ErrorCode, NearError, NearResult, ObjectId, Signature, VerifierTrait};

use crate::{network::DataContext, Stack};

use super::DynamicTunnel;

#[async_trait::async_trait]
pub trait OnRecvMessageCallback: Send + Sync {
    async fn on_callback(
        &self, 
        tunnel: DynamicTunnel, 
        data_context: DataContext
    ) -> NearResult<()>;
}

#[async_trait::async_trait]
pub trait OnSendMessageCallback<DataContext>: Send + Sync {
    async fn on_callback(
        &self, 
        tunnel: DynamicTunnel, 
        data_context: DataContext
    ) -> NearResult<()>;
}

pub(crate) struct TunnelVerifier {
    remote: DeviceObject,
}

impl TunnelVerifier {
    pub async fn new(stack: &Stack, remote_id: &ObjectId) -> NearResult<Self> {
        Ok(Self {
            remote: {
                stack.cacher_manager()
                    .get(&remote_id)
                    .await
                    .ok_or_else(|| {
                        NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("not found {}", remote_id))
                    })?
            },
        })
    }
}

#[async_trait::async_trait]
impl VerifierTrait for TunnelVerifier {
    async fn verify(&self, data: &[u8], sign: &Signature) -> NearResult<()> {
        let tunnel_key = 
            self.remote.desc().public_key()
                .ok_or_else(|| {
                    let error_string = format!("{} missing public key.", self.remote.object_id());
                    log::warn!("{error_string}");
                    NearError::new(ErrorCode::NEAR_ERROR_MISSING_DATA, error_string)
                })?;

        tunnel_key.verify(data, sign)
    }
}
