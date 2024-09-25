
use std::{any::Any, sync::Arc, };

use near_base::*;

use super::{PackageHeader, package_header::PackageHeaderExt, MajorCommand, };

// pub trait PackageBodyTrait: Send + Sync {
//     fn major_command(&self) -> MajorCommand;
// }
pub trait PackageBodyTrait: Send + Sync + Serialize {
    fn version() -> u8;
}

pub type DynamicPackageBody = Box<dyn Any + Send + Sync>;

#[derive(Clone)]
enum ContextFlag {
    Init,
    SerializeHeadExt {
        headext_capacity: usize,
    },
    SerializedBody {
        headext_capacity: usize,
        body_capacity: usize,
    },
    SerializedSign {
        #[allow(unused)]
        headext_capacity: usize,
        #[allow(unused)]
        body_capacity: usize,
        #[allow(unused)]
        sign_capacity: usize,
    },
    Finished,
}

pub struct Context {
    text: Vec<u8>,
    flag: ContextFlag,
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            flag: self.flag.clone(),
        }
    }
}

impl Context {
    pub(super) fn init() -> Self {
        Self {
            text: vec![],
            flag: ContextFlag::Init,
        }
    }

    pub(super) fn serialize_headext(&mut self, head_ext: &PackageHeaderExt) -> NearResult<()> {
        match self.flag {
            ContextFlag::Init => {
                let head_ext_capacity = head_ext.raw_capacity();
                let text = {
                    self.text.resize(head_ext_capacity, 0u8);
                    &mut self.text.as_mut_slice()
                };

                let _ = head_ext.serialize(text)?;

                self.flag = ContextFlag::SerializeHeadExt { headext_capacity: head_ext_capacity };

                Ok(())
            }
            _ => { unreachable!() }
        }

    }

    pub(super) fn serialize_body<B>(&mut self, body: &B) -> NearResult<()> 
    where B: Serialize {
        match self.flag {
            ContextFlag::SerializeHeadExt { headext_capacity } => {
                let body_capacity = body.raw_capacity();
                let text = {
                    self.text.resize(headext_capacity + body_capacity, 0u8);
                    &mut self.text.as_mut_slice()[(headext_capacity)..]
                };

                let _ = body.serialize(text)?;
                // let begin_buf = &mut self.stream[(head_capacity+headext_capacity)..];
                // let begin_buf_len = begin_buf.len();
                // let body_end_buf = body.serialize(begin_buf)?;

                self.flag = ContextFlag::SerializedBody { headext_capacity, body_capacity };

                Ok(())
            }
            _ => { unreachable!() }
        }
    }

    pub(super) async fn serialize_sign(&mut self, signer: Option<&impl SignerTrait>) -> NearResult<Option<Signature>> {
        match self.flag {
            ContextFlag::SerializedBody { headext_capacity, body_capacity } => {
                let data = {
                    if let Some(signer) = signer {
                        let signature_data = 
                            &self.text.as_slice()[headext_capacity..(headext_capacity+body_capacity)];
    
                        Some(signer.sign(signature_data).await?)
                    } else {
                        None
                    }
                };

                let data_capacity = data.raw_capacity();
                let text = {
                    self.text.resize(headext_capacity + body_capacity + data_capacity, 0u8);
                    &mut self.text.as_mut_slice()[(headext_capacity + body_capacity)..]
                };

                let _ = data.serialize(text)?;

                self.flag = ContextFlag::SerializedSign { headext_capacity, body_capacity, sign_capacity: data_capacity };

                Ok(data)
            }
            _ => { unreachable!() }
        }
    }

    // pub(super) async fn verify_signature(&mut self, sign_data: &Signature, verifer: &impl VerifierTrait) -> bool {
    //     match self.flag {
    //         ContextFlag::SerializedBody { head_capacity, headext_capacity, body_capacity } => {
    //             verifer.verify(&self.stream[(head_capacity+headext_capacity)..(head_capacity+headext_capacity+body_capacity)], sign_data).await
    //         }
    //         _ => { unreachable!() }
    //     }
    // }

    pub(super) fn finish(&mut self) -> ContextResult<'_> {
        match self.flag {
            ContextFlag::SerializedSign { headext_capacity, body_capacity, sign_capacity } => {
                self.flag = ContextFlag::Finished;
                let text = self.text.as_slice();
                ContextResult {
                    head_ext: &text[..headext_capacity],
                    remain_data: &text[headext_capacity..(headext_capacity+body_capacity+sign_capacity)],
                }
                // self.text
            },
            _ => { unreachable!() }
        }
    }

}

pub struct ContextResult<'a> {
    pub head_ext: &'a [u8],
    pub remain_data: &'a [u8],
}

/// in header, the length contain Body + Signature's length
pub struct DynamicPackage {
    head: PackageHeader,
    head_ext: PackageHeaderExt,
    body: DynamicPackageBody,
    sign_data: Option<Signature>,
}

impl std::fmt::Display for DynamicPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.head.major_command() {
            MajorCommand::Request => {
                write!(f, "Request: head-ext: {}, sequence: {}, timestamp: {}, command: {} topic: {:?}", 
                       self.head_ext,
                       self.head.sequence(),
                       self.head.timestamp(),
                       self.head.major_command(),
                       self.head_ext.topic()) 
            }
            _ => {
                write!(f, "Response: head-ext: {}, sequence: {}, timestamp: {}, command: {}", 
                       self.head_ext,
                       self.head.sequence(),
                       self.head.timestamp(),
                       self.head.major_command()) 
            }
        }
    }
}

impl<B> From<(PackageHeader, PackageHeaderExt, B, Option<Signature>)> for DynamicPackage
where B: 'static + PackageBodyTrait {
    fn from(context: (PackageHeader, PackageHeaderExt, B, Option<Signature>)) -> Self {
        let (head, head_ext, body, sign_data) = context;

        Self {
            head,
            head_ext,
            body: Box::new(body),
            sign_data,
        }
    }
}

impl From<(PackageHeader, PackageHeaderExt, DynamicPackageBody, Option<Signature>)> for DynamicPackage {
    fn from(context: (PackageHeader, PackageHeaderExt, DynamicPackageBody, Option<Signature>)) -> Self {
        let (head, head_ext, body, sign_data) = context;

        Self {
            head,
            head_ext,
            body,
            sign_data,
        }
    }
}

// impl From<(PackageHeader, PackageHeaderExt, DynamicPackageBody, Option<Signature>)> for DynamicPackage {
//     fn from(context: (PackageHeader, PackageHeaderExt, DynamicPackageBody, Option<Signature>)) -> Self {
//         let (head, head_ext, body, sign_data) = context;

//         Self {
//             head,
//             head_ext,
//             body,
//             sign_data,
//         }
//     }
// }

impl DynamicPackage {
    #[inline]
    #[allow(unused)]
    pub fn as_head(&self) -> &PackageHeader {
        &self.head
    }

    #[inline]
    #[allow(unused)]
    pub fn as_headext(&self) -> &PackageHeaderExt {
        &self.head_ext
    }

    #[allow(unused)]
    pub fn verify(&self, _verifer: &impl VerifierTrait) -> bool {
        if let Some(_sign_data) = &self.sign_data {
            // let mut text = [0u8; MTU];
            // let mut cx = Context::init(&mut text);

            // let r = 
            // cx.serialize_body(&self.body)
            //     .map_err(| _ | false );

            // if cx.serialize_body(self.package_head.command(), &self.package_body)
            //      .map_or(false, |_| true) {
            //     cx.verify_signature(&sign_data, verifer).await
            // } else {
            //     false
            // }
            false
        } else {
            true
        }
    }

    pub fn split<T: 'static + Default>(self) -> (PackageHeader, PackageHeaderExt, T, Option<Signature>) {
        let mut body = self.body.downcast::<T>().unwrap();
        let mut data = T::default();

        std::mem::swap(body.as_mut(), &mut data);

        (self.head, 
         self.head_ext, 
         data,
         self.sign_data)
    }

    // pub fn split(self) -> (PackageHeader, PackageHeaderExt, DynamicPackageBody, Option<Signature>) {
    //     (self.head, self.head_ext, self.body, self.sign_data)
    // }

}
/*

impl From<((ObjectId, Option<ObjectId>, SequenceValue), Box<dyn PackageBodyTrait>)> for DynamicPackage {
    fn from(context: ((ObjectId, Option<ObjectId>, SequenceValue), Box<dyn PackageBodyTrait>)) -> Self {
        let ((from, to, seq), body) = context;
        let head = PackageHeader::new(body.command(), seq, from, to);

        Self {
            package_head: head,
            package_body: Box::new(body),
            sign_data: None,
        }
    }
}

// impl From<((ObjectId, Option<ObjectId>, SequenceValue), &Ack)> for DynamicPackage {
//     fn from(context: ((ObjectId, Option<ObjectId>, SequenceValue), &Ack)) -> Self {
//         let ((from, to, seq), body) = context;
//         let head = PackageHeader::new(Command::Exchange.into(), seq, from, to);
//         let body = Box::new(body.clone());

//         Self {
//             package_head: head,
//             package_body: body,
//             sign_data: None,
//         }
//     }
// }

// impl From<((ObjectId, Option<ObjectId>, SequenceValue), &AckAck)> for DynamicPackage {
//     fn from(context: ((ObjectId, Option<ObjectId>, SequenceValue), &AckAck)) -> Self {
//         let ((from, to, seq), body) = context;
//         let head = PackageHeader::new(Command::Exchange.into(), seq, from, to);
//         let body = Box::new(body.clone());

//         Self {
//             package_head: head,
//             package_body: body,
//             sign_data: None,
//         }
//     }
// }

impl DynamicPackage {
    pub async fn serialize<'a>(&mut self, 
                               buf: &'a mut [u8],
                               signer: Option<impl SignerTrait>) -> NearResult<&'a mut [u8]> {
        if buf.len() < PackageHeader::capacity() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "the buf isn't enough space."));
        }

        let mut cx = Context::init(buf);
        cx.serialize_body(self.package_head.command(), &&self.package_body)?;
        cx.serialize_sign(signer).await?;
        let total_size = cx.finish(&mut self.package_head)?;

        Ok(&mut buf[total_size..])
    }

    pub async fn deserialize<'a>(head: PackageHeader, 
                                 buf: &'a [u8]) -> NearResult<(Self, &'a [u8])> {
        let (package, buf) = match head.command().into() {
            Command::Exchange => {
                unimplemented!()
            },
            Command::SynTunnel => {
                SynTunnel::deserialize(buf)
                    .map(| (body, buf) | {
                        (Box::new(body) as DynamicPackageBody, buf)
                    })
            },
            Command::AckTunnel => {
                AckTunnel::deserialize(buf)
                    .map(| (body, buf) | {
                        (Box::new(body) as DynamicPackageBody, buf)
                    })
            }
            Command::AckAck => {
                AckAck::deserialize(buf)
                    .map(| (body, buf) | {
                        (Box::new(body) as DynamicPackageBody, buf)
                    })
            },
            _ => {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_UNKNOWN_PROTOCOL, "unknown protocol"));
            }
        }?;

        let (sign_data, buf) = Option::<Signature>::deserialize(buf)?;

        Ok((Self {
            package_head: head,
            package_body: package,
            sign_data
        }, buf))
    }
}
*/

impl<T> AsRef<T> for DynamicPackage
where T: 'static + PackageBodyTrait + Serialize + Deserialize + Send + Sync{
    fn as_ref(&self) -> &T {
        self.body.downcast_ref::<T>().unwrap()
    }
}

#[derive(Clone)]
pub struct DynamicPackageGuard(Arc<DynamicPackage>);

impl std::ops::Deref for DynamicPackageGuard {
    type Target = DynamicPackage;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
