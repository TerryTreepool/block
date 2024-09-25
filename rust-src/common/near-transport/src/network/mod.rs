
mod data_context;
mod tcp;
mod udp;
mod manager;
mod interface;

use std::collections::VecDeque;

use near_base::{DeviceObject, ErrorCode, NearError, NearResult, ObjectId, };
pub use tcp::Tcp;
pub use udp::Udp;
pub use manager::NetManager;
pub use interface::{Interface, DynamicInterface, 
                    TcpInterface, UdpInterface,
                    PackageDecodeTrait};

use crate::package::{CreateVeriferTrait, DynamicPackage, PackageHeader, PackageHeaderExt, PackageParser};

pub const MTU: usize = 1472;

#[derive(Clone)]
pub struct DataContext {
    pub head: PackageHeader,
    pub head_ext: PackageHeaderExt,
    pub body_data: Vec<u8>,
}

impl std::fmt::Display for DataContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[head: {}, head-ext: {}, body-size: {}]", self.head, self.head_ext, self.body_data.len())
    }
}

// impl TryInto<DynamicPackage> for DataContext {
//     type Error = NearError;

//     fn try_into(self) -> Result<DynamicPackage, Self::Error> {
//         let (package, _) = PackageParser::new(self.head, self.head_ext).parse(&self.body_data)?;

//         Ok(package)
//     }
// }
impl DataContext {
    pub(crate) async fn parse(self, create_verifer: impl CreateVeriferTrait,) -> NearResult<DynamicPackage> {
        let (package, _) = 
            PackageParser::new(self.head, self.head_ext)
                .parse(&self.body_data, create_verifer)
                .await?;
            
        Ok(package)
    }
}

impl near_base::Serialize for DataContext {
    fn raw_capacity(&self) -> usize {
        self.head.raw_capacity() + 
        self.head_ext.raw_capacity() + 
        self.body_data.len()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let cap = self.raw_capacity();
        if buf.len() < cap {
            Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough space."))
        } else {
            Ok(())
        }?;

        let buf = self.head.serialize(buf)?;
        let buf = self.head_ext.serialize(buf)?;
        let buf = {
            let len = self.body_data.len();
            unsafe {
                std::ptr::copy(self.body_data.as_ptr(), buf.as_mut_ptr(), len);
            }
            &mut buf[..len]
        };

        Ok(buf)
    }
}

impl DataContext {

    pub fn merge(mut data_context: VecDeque<Option<DataContext>>) -> Self {

        let first = data_context.pop_front().unwrap().unwrap();

        // build new context
        let head = PackageHeader::default()
                                    .set_major_command(first.head.major_command())
                                    .set_sequence(*first.head.sequence())
                                    .set_timestamp(first.head.timestamp());

        let mut context = DataContext {
            head,
            head_ext: first.head_ext,
            body_data: first.body_data,
        };

        context.merge_one(data_context);

        context
    }

    fn merge_one(&mut self, mut data_context: VecDeque<Option<DataContext>>) {

        if data_context.len() > 0 {
            let first = data_context.pop_front().unwrap().unwrap();
            
            debug_assert!(self.head.sequence() == first.head.sequence(), "head sequence must be the same.");

            let body_data_len = self.body_data.len();
            let merged_body_data_len = first.body_data.len();
            let new_body_len = body_data_len + merged_body_data_len;
    
            self.body_data.resize(new_body_len, 0);
            unsafe {
                std::ptr::copy(first.body_data.as_ptr(), self.body_data[body_data_len..].as_mut_ptr(), merged_body_data_len);
            }

            self.merge_one(data_context)
        }
    }
}

#[async_trait::async_trait]
pub trait UdpPackageEventTrait<Context> {
    fn on_connected(&self, interface: UdpInterface, remote: &DeviceObject, _: Context);
    async fn on_udp_package(&self, interface: UdpInterface, package: DataContext, _: Context) -> NearResult<()>;
}

#[async_trait::async_trait]
pub trait TcpPackageEventTrait {
    fn on_connected(&self, interface: TcpInterface, remote: &DeviceObject);
    fn on_closed(&self, interface: &TcpInterface, remote: &ObjectId);
    async fn on_tcp_package(&self, interface: TcpInterface, package: DataContext) -> NearResult<()>;
}

pub trait TcpStateEventTrait {
    fn on_closed(&self, interface: &TcpInterface);
}
