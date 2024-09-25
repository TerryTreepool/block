
use std::path::PathBuf;

use near_base::{NearResult, NearError, ErrorCode, PrivateKey, FileDecoder, DeviceObject, ObjectTypeCode, Endpoint};
use near_core::{path_utils::get_bin_path, get_data_path};

#[cfg(target_os="windows")]
static EXE_EXTENSION: &str = "exe";

#[cfg(any(target_os="android", target_os="ios", target_os="linux", target_os="macos", target_os="unix"))]
static EXE_EXTENSION: &str = "";

trait CheckTrait {
    fn check_desc(&self) -> NearResult<()>;
    fn check_bin(&self) -> NearResult<()>;
    fn check_key(&self) -> NearResult<()>;
}

pub struct CoreServiceCheck {
    bin: PathBuf,
    desc: PathBuf,
    key: PathBuf,
}

impl std::default::Default for CoreServiceCheck {
    fn default() -> Self {
        Self {
            bin: PathBuf::new().with_file_name("core-service").with_extension(EXE_EXTENSION),
            desc: PathBuf::new().with_file_name("core-service").with_extension(DESC_SUFFIX_NAME),
            key: PathBuf::new().with_file_name("core-service").with_extension(KEY_SUFFIX_NAME),
        }
    }
}

impl CheckTrait for CoreServiceCheck {
    fn check_bin(&self) -> NearResult<()> {
        let path = get_bin_path().join(&self.bin);

        if !path.exists() {
            Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("not found {}", self.bin.display())))
        } else {
            Ok(())
        }?;

        Ok(())
    }

    fn check_desc(&self) -> NearResult<()> {
        let desc_path = get_data_path().join(&self.desc);

        if desc_path.exists() {
            Ok(())
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("not found {}", self.desc.display())))
        }?;

        let desc = DeviceObject::decode_from_file(&desc_path)?;

        match desc.object_id().object_type_code()? {
            ObjectTypeCode::Device(_) => Ok(()),
            _ => {
                Err(NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, "object type code isnot Device."))
            }
        }?;

        // endpoints
        let mut endpoints: Vec<&Endpoint> = 
            desc.body().content().endpoints()
                .iter()
                .filter(| &addr | {
                    if addr.is_ipv4() && !addr.is_static_wan() {
                        return true;
                    } else {
                        return false;
                    }
                })
                .collect();

        if endpoints.len() > 0 {
            // check ip
            near_util::get_if_sockaddrs()?
                .iter()
                .filter(| it | {
                    it.is_ipv4()
                })
                .for_each(| it | {
                    // if it.is_ipv4() {
                    //     if protocol.contains(Protocol::ProtocolTcp) {
                    //         endpoints.push(
                    //             {
                    //                 if let Some(ipaddr) = ipaddr.as_ref() {
                    //                     Endpoint::default_tcp(SocketAddr::new(ipaddr.clone())).set_static_wan(true)
                    //                 } else {
                    //                     Endpoint::default_tcp(SocketAddr::new(ipaddr.clone()))
                    //                 }
                    //             },
                    //             port)
                    //     }
                    //     if protocol.contains(Protocol::ProtocolUdp) {
                    //         endpoints.push(Endpoint::default_udp(SocketAddr::new({
                    //             match ipaddr.as_ref() {
                    //                 Some(addr) => IpAddr::from(addr.clone()),
                    //                 None => it.clone(),
                    //             }
                    //         },
                    //         port)))
                    //     }
                    // } else {
                    //     if protocol.contains(Protocol::ProtocolTcp) {
                    //         endpoints.push(Endpoint::default_tcp(SocketAddr::new(it.clone(), port)))
                    //     }
                    //     if protocol.contains(Protocol::ProtocolUdp) {
                    //         endpoints.push(Endpoint::default_udp(SocketAddr::new(it.clone(), port)))
                    //     }
                    // }
                });

        }

        Ok(())
    }

    fn check_key(&self) -> NearResult<()> {
        let key_path = get_data_path().join(&self.key);

        if key_path.exists() {
            Ok(())
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("not found {}", self.key.display())))
        }?;

        let _ = PrivateKey::decode_from_file(&key_path)?;

        Ok(())
    }
}

fn main() {
    // check core-service
    check_core_service().expect("failed check core-service: ");
}
