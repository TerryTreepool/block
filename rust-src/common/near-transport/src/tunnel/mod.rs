
pub mod container;
pub mod manager;
pub mod tunnel;
pub mod tunnel_state;
pub mod tcp;
pub mod udp;

mod message;
mod p;

pub use manager::{Manager as TunnelManager, Config as TunnelManagerConfig, };
pub use container::Config as TunnelContainerConfig;
use near_base::{ObjectId, Endpoint, NearResult};
pub use tunnel::{TunnelStateTrait, DynamicTunnel};
pub use tcp::Config as TcpConfig;
pub use udp::Config as UdpConfig;

#[derive(Clone)]
pub struct Config {
    pub manager: TunnelManagerConfig,
    pub container: TunnelContainerConfig,
    pub tcp_config: TcpConfig,
    pub udp_config: UdpConfig,
}

#[async_trait::async_trait]
pub trait TunnelEventTrait: Sync + Send {
    /// The target corresponds to all tunnel interrupts. 
    /// This interface will be called to notify the upper layer
    async fn on_reconnect(&self, ep: Endpoint, target: &ObjectId) -> NearResult<()>;

}

#[async_trait::async_trait]
pub trait PostMessageTrait<DataContext> {
    type R;
    async fn post_message(&self, context: DataContext) -> NearResult<Self::R>;
}
