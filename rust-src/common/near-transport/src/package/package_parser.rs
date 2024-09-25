
use near_base::*;
use super::{body::*, package::DynamicPackageBody, DynamicPackage, PackageHeader, PackageHeaderExt  
    };

#[async_trait::async_trait]
pub(crate) trait CreateVeriferTrait {
    async fn create_verifer_obj(&self, requestor: &ObjectId) -> NearResult<Box<dyn VerifierTrait>>;
}

pub struct PackageParser {
    head: PackageHeader,
    head_ext: PackageHeaderExt,
}

impl PackageParser {
    pub fn new(
        head: PackageHeader, 
        head_ext: PackageHeaderExt,
    ) -> Self {
        Self { head, head_ext, }
    }

    pub(crate) async fn parse<'a>(
        self, 
        data: &'a [u8],
        create_verifer: impl CreateVeriferTrait,
    ) -> NearResult<(DynamicPackage, &'a [u8])> {
        let head = self.head;
        let head_ext = self.head_ext;

        let (body, sign_data) = match head.major_command() {
            super::MajorCommand::Exchange => 
                Exchange::deserialize(data)
                    .map(|(body, buf)| {
                        (Box::new(body) as DynamicPackageBody, buf)
                    }),
            super::MajorCommand::AckAckTunnel => 
                AckAckTunnel::deserialize(data)
                    .map(| (body, buf) | {
                        (Box::new(body) as DynamicPackageBody, buf)
                    }),
            super::MajorCommand::AckTunnel => 
                AckTunnel::deserialize(data)
                    .map(| (body, buf) | {
                        (Box::new(body) as DynamicPackageBody, buf)
                    }),
            super::MajorCommand::Stun => 
                StunReq::deserialize(data)
                    .map(| (body, buf) | {
                        (Box::new(body) as DynamicPackageBody, buf)
                    }),
            // super::MajorCommand::Ping => 
            //     Ping::deserialize(data)
            //         .map(| (body, buf) | {
            //             (Box::new(body) as DynamicPackageBody, buf)
            //         }),
            // super::MajorCommand::PingResp => 
            //     PingResp::deserialize(data)
            //         .map(| (body, buf) | {
            //             (Box::new(body) as DynamicPackageBody, buf)
            //         }),
            // super::MajorCommand::CallCommand => {
            //     let minor_command = 
            //         CallSubCommand::from_str(
            //             head_ext.topic()
            //                 .ok_or_else(|| NearError::new(ErrorCode::NEAR_COMMAND_MINOR, "missing topic"))?
            //                 .as_str()
            //             )?;

            //     match minor_command {
            //         CallSubCommand::Call => {
            //             CallReq::deserialize(data)
            //                 .map(| (body, buf) | {
            //                     (Box::new(body) as DynamicPackageBody, buf)
            //                 })
            //         },
            //         CallSubCommand::CallResp => {
            //             CallResp::deserialize(data)
            //                 .map(| (body, buf) | {
            //                     (Box::new(body) as DynamicPackageBody, buf)
            //                 })
            //         },
            //         CallSubCommand::Called => {
            //             CalledReq::deserialize(data)
            //                 .map(| (body, buf) | {
            //                     (Box::new(body) as DynamicPackageBody, buf)
            //                 })
            //         },
            //         CallSubCommand::CalledResp => {
            //             CalledResp::deserialize(data)
            //                 .map(| (body, buf) | {
            //                     (Box::new(body) as DynamicPackageBody, buf)
            //                 })
            //         },
            //     }
            // }
            super::MajorCommand::Ack => 
                Ack::deserialize(data)
                    .map(| (body, buf) | {
                        (Box::new(body) as DynamicPackageBody, buf)
                    }),
            super::MajorCommand::AckAck => 
                AckAck::deserialize(data)
                    .map(| (body, buf) | {
                        (Box::new(body) as DynamicPackageBody, buf)
                    }),
            super::MajorCommand::Request => 
                Data::deserialize(data)
                    .map(| (body, buf) | {
                        (Box::new(body) as DynamicPackageBody, buf)
                    }),
            super::MajorCommand::Response => 
                Data::deserialize(data)
                    .map(| (body, buf) | {
                        (Box::new(body) as DynamicPackageBody, buf)
                    }),
            super::MajorCommand::None => { Err(NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN_PROTOCOL, "unknown protocol")) }
        }?;
        
        let (signature, remain_data) = Option::<Signature>::deserialize(sign_data)?;

        // check signature
        if let Some(signature_ref) = signature.as_ref() {
            let data_len = data.len();
            let siga_data_len = sign_data.len();

            create_verifer.create_verifer_obj(head_ext.requestor())
                .await?
                .verify(&data[..data_len-siga_data_len], signature_ref)
                .await
        } else {
            Ok(())
        }?;

        Ok((DynamicPackage::from((head, head_ext, body, signature)), remain_data))
    }
}

