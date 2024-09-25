
#![allow(non_upper_case_globals)]

use std::{net::{IpAddr, SocketAddr}, path::PathBuf, str::FromStr, };

use clap::Arg;

use extention::{ExtentionBodyContent, ExtentionDescContent};
use near_base::{device::*, *};
use near_core::*;
use near_util::{CORE_STACK_PORT, DESC_SUFFIX_NAME, KEY_SUFFIX_NAME};
use people::{PeopleBodyContent, PeopleDescContent, PeopleObject};

lazy_static::lazy_static!{
    pub static ref owner_arg_command: Arg<'static> =
        Arg::with_name("owner")
            .long("owner")
            .takes_value(true)
            .help("Owner id");

    pub static ref area_arg_command: Arg<'static> =
        Arg::with_name("area")
            .long("area")
            .takes_value(true)
            .help("Object area info, if not set, will calc base ip. format [county:carrier:city:inner]");

    pub static ref ipprotocol_arg_command: Arg<'static> =
        Arg::with_name("ip_protocol")
            .long("protocol")
            .takes_value(true)
            .default_value("tcp")
            .possible_values(&["tcp", "udp", "all"])
            .help("Object network protocol");

    pub static ref iptype_arg_command: Arg<'static> =
        Arg::with_name("ip_addr_type")
            .long("iptype")
            .takes_value(true)
            .default_value("ipv4")
            .possible_values(&["ipv4", "ipv6", "all"])
            .help("Object network address type");

    pub static ref ipaddr_arg_command: Arg<'static> =
        Arg::with_name("ip_addr")
            .long("ipaddr")
            .short('H')
            .takes_value(true)
            .help("Object network address");

    pub static ref ipport_arg_command: Arg<'static> =
        Arg::with_name("port")
            .long("port")
            .short('P')
            .default_value("13456")
            .takes_value(true)
            .help("Object network port, you can listen custom port. default is 13456");

    pub static ref output_path_arg_command: Arg<'static> =
        Arg::with_name("output path")
            .long("output")
            .short('O')
            .takes_value(true)
            .help("Output file path. if not set, will generate in ${{NEAR_HOME}}/data");

    pub static ref userdata_arg_command: Arg<'static> =
        Arg::with_name("userdata")
            .long("userdata")
            .takes_value(true)
            .help("User's data");

    pub static ref extention_name_arg_command: Arg<'static> =
        Arg::with_name("name")
            .long("name")
            .takes_value(true)
            .help("Extended service name in NearOS");

    pub static ref subscribe_arg_command: Arg<'static> =
        Arg::with_name("subscribe messages")
            .long("messages")
            .takes_value(true)
            .help("Extended service subscribe messages in NearOS, eg: /near/message1:/near/message2");

    pub static ref depended_desc_arg_command: Arg<'static> =
        Arg::with_name("depended desc")
            .long("desc")
            .takes_value(true)
            .help("You need depended desc.");

    pub static ref core_service_arg_command: Arg<'static> =
        Arg::with_name("core-service")
            .long("core")
            .takes_value(true)
            .help("The path where the core-service is located in NearOS. if not set, will found in ${{NEAR_HOME}}/data");

    pub static ref pktype_arg_command: Arg<'static> =
        Arg::with_name("pktype")
            .long("pktype")
            .default_value("rsa2048")
            .possible_values(&["rsa1024", "rsa2048"])
            .help("Private key type");

    pub static ref service_type_command: Arg<'static> =
        Arg::with_name("service type")
            .long("ctype")
            .takes_value(true)
            .help("service run type: coturn-miner");

    pub static ref people_name_arg_command: Arg<'static> =
        Arg::with_name("name")
            .long("name")
            .takes_value(true)
            .help("Your name in NearOS can be seen by your friends.");

}

pub enum ServiceObjectType {
    Service(u8),
    Device(u8),
    Extention,
    People,
}

#[derive(Clone, Copy)]
enum PkType {
    RSA1024,
    RSA2048,
}

impl TryFrom<&str> for PkType {
    type Error = NearError;

    fn try_from(value: &str) -> NearResult<Self> {
        match value {
            "rsa1024" => Ok(Self::RSA1024),
            "rsa2048" => Ok(Self::RSA2048),
            _ => Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "{value} undefined"))
        }
    }
}

impl PkType {
    pub fn build(self) -> NearResult<PrivateKey> {
        match self {
            Self::RSA1024 => PrivateKey::generate_rsa1024(),
            Self::RSA2048 => PrivateKey::generate_rsa2048(),
        }
    }
}

#[repr(u32)]
#[enumflags2::bitflags]
#[derive(Clone, Copy)]
enum Protocol {
    ProtocolTcp = 1 << 1,
    ProtocolUdp = 1 << 2,
}

pub enum NetworkType {
    Ipv4,
    Ipv6,
    IpAll,
}

pub enum DescToolBuilderOp {
    Create,
    Modify,
}

pub struct DescToolBuilder<'a> {

    pub op: DescToolBuilderOp,
    pub device_type: ServiceObjectType,
    pub owner: Option<&'a str>,
    pub area: Option<&'a str>,
    pub pktype: Option<&'a str>,
    pub output: Option<PathBuf>,
    pub userdata: Option<&'a str>,
    pub port: Option<&'a str>,
    pub host: Option<&'a str>,
    pub protocol: Option<&'a str>,
    pub network: Option<&'a str>,
    pub name: Option<&'a str>,
    pub depended_desc: Option<&'a str>,
    pub subscribe_messages: Option<&'a str>,
}

impl<'a> DescToolBuilder<'a> {
    pub fn build(self) -> NearResult<ServiceObjectDescParams<'a>> {

        let owner = 
            self.owner.map(| owner | {
                ObjectId::from_str(owner).expect("invalid owner")
            });
        let area = 
            self.area.map(| area | Area::from_str(area).expect("invalid area"));
        let output = self.output;
        let userdata = self.userdata.map(| userdata | userdata.to_vec().expect("invalid userdata"));
        let pktype = 
            self.pktype.map(| pktype | {
                let key: PkType = pktype.try_into().expect("invalid key");
                key.build().expect("failed gen key")
            });
        let host = 
            self.host.map(| v | {
                std::net::Ipv4Addr::from_str(v).expect("invalid ip-addr")
            });
        let port = 
            self.port.map(| port | {
                port.parse::<u16>().expect("invalid ip-port")
            })
            .unwrap_or(CORE_STACK_PORT);
        let protocol = 
            match self.protocol.unwrap_or("all") {
                "tcp" => enumflags2::make_bitflags!(Protocol::{ProtocolTcp}),
                "udp" => enumflags2::make_bitflags!(Protocol::{ProtocolUdp}),
                _ => enumflags2::make_bitflags!(Protocol::{ProtocolTcp | ProtocolUdp}),
            };
        let iptype = 
            match self.network.unwrap_or("ipv4") {
                "ipv4" => NetworkType::Ipv4,
                "ipv6" => NetworkType::Ipv6,
                _ => NetworkType::IpAll,
            };
        let name = self.name;

        let endpoints = {
            let mut endpoints = vec![];

            near_util::get_if_sockaddrs().expect("get ip address")
                .iter()
                .filter(| it | {
                    match iptype {
                        NetworkType::Ipv4 => it.is_ipv4(),
                        NetworkType::Ipv6 => it.is_ipv6(),
                        NetworkType::IpAll => true,
                    }
                })
                .for_each(| it | {
                    if it.is_ipv4() {
                        if protocol.contains(Protocol::ProtocolTcp) {
                            endpoints.push(
                                if let Some(host) = host.as_ref() {
                                    Endpoint::default_tcp(SocketAddr::new(IpAddr::from(host.clone()), port)).set_static_wan(true)
                                } else {
                                    Endpoint::default_tcp(SocketAddr::new(it.clone(), port))
                                }
                            )
                        }
                        if protocol.contains(Protocol::ProtocolUdp) {
                            endpoints.push(
                                if let Some(host) = host.as_ref() {
                                    Endpoint::default_udp(SocketAddr::new(IpAddr::from(host.clone()), port)).set_static_wan(true)
                                } else {
                                    Endpoint::default_udp(SocketAddr::new(it.clone(), port))
                                }
                            )
                        }
                    } else {
                        if protocol.contains(Protocol::ProtocolTcp) {
                            endpoints.push(Endpoint::default_tcp(SocketAddr::new(it.clone(), port)))
                        }
                        if protocol.contains(Protocol::ProtocolUdp) {
                            endpoints.push(Endpoint::default_udp(SocketAddr::new(it.clone(), port)))
                        }
                    }
                });

            endpoints
        };

        Ok(ServiceObjectDescParams {
            op: self.op,
            device_type: self.device_type,
            owner,
            area,
            output,
            userdata,
            pktype,
            endpoints: Some(endpoints),
            name,
            depended_desc: 
                if let Some(depended_desc) = self.depended_desc {
                    Some(get_data_path().join(format!("{depended_desc}.{DESC_SUFFIX_NAME}")))
                } else {
                    None
                }, 
            subscribe_messages: self.subscribe_messages,
        })

    }
}

pub struct ServiceObjectDescParams<'a> {
    op: DescToolBuilderOp,

    device_type: ServiceObjectType,
    owner: Option<ObjectId>,
    area: Option<Area>,
    output: Option<PathBuf>,
    userdata: Option<Vec<u8>>,
    pktype: Option<PrivateKey>,
    endpoints: Option<Vec<Endpoint>>,
    name: Option<&'a str>,

    /// extention property
    depended_desc: Option<PathBuf>,
    subscribe_messages: Option<&'a str>,
}

impl ServiceObjectDescParams<'_> {
    pub fn build(self) -> NearResult<()> {

        match &self.device_type {
            ServiceObjectType::Device(_) => self.build_device_desc(),
            ServiceObjectType::Service(_) => self.build_service_desc(),
            ServiceObjectType::Extention => self.build_extention_desc(),
            ServiceObjectType::People => self.build_people_desc(),
        }

    }

    fn build_people_desc(self) -> NearResult<()> {
        let op = self.op;
        let device_type = self.device_type;
        let _owner = self.owner;
        let _area = self.area;
        let output = self.output.unwrap_or(get_data_path());
        let userdata = self.userdata;
        let pktype = self.pktype;
        let _endpoints = self.endpoints;
        let name = self.name.unwrap_or("BM");

        if let ServiceObjectType::People = device_type {
            Ok(())
        }
        else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, "not people type"))
        }?;

        let (desc, private_key) = 
            match op {
                DescToolBuilderOp::Create => {
                    let pktype = pktype.expect("missing private key.");
                    let desc = 
                        ObjectBuilder::new(
                            PeopleDescContent::new(), 
                            PeopleBodyContent::default()
                        )
                        .update_desc(| desc | {
                            desc.set_public_key(pktype.public());
                        })
                        .update_body(| body | {
                            body.mut_body()
                                .set_name(Some(name))
                                .set_userdata(userdata);
                        })
                        .build()?;

                    (desc, Some(pktype))
                }
                DescToolBuilderOp::Modify => {
                    // let output = output.as_ref().expect("missing modify desc path");
                    let output = output.join(format!("{name}.{DESC_SUFFIX_NAME}"));
                    let mut desc = PeopleObject::decode_from_file(output.as_path())?;

                    if let Some(userdata) = userdata {
                        desc.mut_body().set_user_data(Some(userdata));
                    }
                    desc.mut_body().mut_content().set_name(Some(name));
                    desc.mut_body().set_update_time();

                    (desc, None)
                }
            };

        ObjectDescOutput{
            output_path: Some(output),
            desc,
            desc_name: name.to_owned(),
            private_key: private_key,
        }.output()
    }

    fn build_service_desc(self) -> NearResult<()> {
        let op = self.op;
        let device_type = self.device_type;
        let owner = self.owner;
        let area = self.area;
        let output = self.output.unwrap_or(get_data_path());
        let userdata = self.userdata;
        let pktype = self.pktype;
        let endpoints = self.endpoints;
        let name = self.name.expect("missing service name");
        
        let service_object_sub_type = 
            if let ServiceObjectType::Service(v) = &device_type {
                *v
            } else {
                panic!("don't ServiceObjectType::Service type.")
            };

            let (desc, private_key) = 
            match op {
                DescToolBuilderOp::Create => {
                    let pktype = pktype.expect("missing private key.");
                    let endpoints = endpoints.expect("missing endpoints");

                    let desc = 
                        ObjectBuilder::new(
                            DeviceDescContent::with_service(service_object_sub_type),
                            DeviceBodyContent::default()
                        )
                        .update_desc(| desc | {
                            desc.set_owner(owner);
                            desc.set_area(area);
                            desc.set_public_key(pktype.public());
                        })
                        .update_body(|body| {
                            body.mut_body().set_name(Some(name));
                            if let Some(userdata) = userdata {
                                body.mut_body().set_userdata(userdata);
                            }
                            body.mut_body().set_endpoints(endpoints);
                            // body.mut_body().set_endpoints(endpoints)
                        })
                        .build()?;

                    (desc, Some(pktype))
                }
                DescToolBuilderOp::Modify => {
                    let output = output.join(format!("{name}.{DESC_SUFFIX_NAME}"));
                    let mut desc = DeviceObject::decode_from_file(output.as_path())?;

                    // check
                    if let ObjectTypeCode::Service(v) = desc.object_id().object_type_code().expect("invalid service data") {
                        if v == ServiceObjectSubCode::OBJECT_TYPE_SERVICE_COTURN_MINER as u8 {}
                        else {
                            panic!("invalid service sub code");
                        }
                    }

                    if let Some(userdata) = userdata {
                        desc.mut_body().set_user_data(Some(userdata));
                    }
                    if let Some(endpoints) = endpoints {
                        desc.mut_body().mut_content().set_endpoints(endpoints);
                    }
                    desc.mut_body().set_update_time();

                    (desc, None)
                }
            };

        // let name = 
        //     name.map(| name | name.to_string())
        //         .unwrap_or(desc.object_id().to_string());

        ObjectDescOutput{
            output_path: Some(output),
            desc,
            desc_name: name.to_owned(),
            private_key: private_key,
        }.output()
    }

    fn build_device_desc(self) -> NearResult<()> {
        let op = self.op;
        let device_type = self.device_type;
        let owner = self.owner;
        let area = self.area;
        let output = self.output.unwrap_or(get_data_path());
        let userdata = self.userdata;
        let pktype = self.pktype;
        let endpoints = self.endpoints;
        let name = self.name.expect("missing device name");

        let service_object_sub_type = 
            if let ServiceObjectType::Device(v) = &device_type {
                *v
            } else {
                panic!("don't ServiceObjectType::Device type.")
            };

        let (desc, private_key) = 
            match op {
                DescToolBuilderOp::Create => {
                    let pktype = pktype.expect("missing private key.");
                    let endpoints = endpoints.expect("missing endpoints");

                    let desc = 
                        ObjectBuilder::new(
                            DeviceDescContent::with_device(service_object_sub_type),
                            DeviceBodyContent::default()
                        )
                        .update_desc(| desc | {
                            desc.set_owner(owner);
                            desc.set_area(area);
                            desc.set_public_key(pktype.public());
                        })
                        .update_body(|body| {
                            body.mut_body().set_name(Some(name));
                            if let Some(userdata) = userdata {
                                body.mut_body().set_userdata(userdata);
                            }
                            body.mut_body().set_endpoints(endpoints);
                        })
                        .build()?;

                    (desc, Some(pktype))
                }
                DescToolBuilderOp::Modify => {
                    let output = output.join(format!("{name}.{DESC_SUFFIX_NAME}"));
                    let mut desc = DeviceObject::decode_from_file(output.as_path())?;

                    // check
                    if let ObjectTypeCode::Device(v) = desc.object_id().object_type_code().expect("invalid device data") {
                        if v != DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8 {
                            panic!("invalid device sub code");
                        }
                    }

                    if let Some(userdata) = userdata {
                        desc.mut_body().set_user_data(Some(userdata));
                    }
                    if let Some(endpoints) = endpoints {
                        desc.mut_body().mut_content().set_endpoints(endpoints);
                    }
                    desc.mut_body().set_update_time();

                    (desc, None)
                }
            };

        // // let name = 
        // //     name.map(| name | name.to_string())
        // //         .unwrap_or(desc.object_id().to_string());

        ObjectDescOutput{
            output_path: Some(output),
            desc,
            desc_name: name.to_owned(),
            private_key: private_key,
        }.output()
    }

    fn build_extention_desc(self) -> NearResult<()> {
        let op = self.op;
        let device_type = self.device_type;
        let _owner = self.owner;
        let _area = self.area;
        let output = self.output.unwrap_or(get_data_path());
        let userdata = self.userdata;
        let _pktype = self.pktype;
        let _endpoints = self.endpoints;
        let name = self.name.expect("You must have extention name.");
        let depended_desc = self.depended_desc;

        if let ServiceObjectType::Extention = device_type {
            Ok(())
        }
        else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_UNMATCH, "not extention type"))
        }?;

        let subscribe_messages = 
            self.subscribe_messages.map(| messages | {
                let messages: Vec<&str> = messages.split(':').collect();
                messages
            });

        let extention_name = name;

        let desc = 
            match op {
                DescToolBuilderOp::Create => {
                    let depended_desc = depended_desc.expect("missing depended desc, you need core-service desc");
            
                    let depended_desc = {
                        if !depended_desc.exists() {
                            return Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("{} isn't exist.", depended_desc.display())));
                        } else {
                            DeviceObject::decode_from_file(depended_desc.as_path())?
                        }
                    };

                    ObjectBuilder::new(
                        ExtentionDescContent::default(), 
                        ExtentionBodyContent::default()
                    )
                    .update_desc(| desc |{
                        desc.set_owner(Some(depended_desc.object_id().clone()));
                        desc.mut_desc().set_extention_name(extention_name);
                    })
                    .update_body(| body | {
                        if let Some(messages) = subscribe_messages {
                            body.mut_body().set_subscribe_message_group(&messages);
                        }
                    })
                    .build()?
                }
                DescToolBuilderOp::Modify => {
                    let output = output.join(format!("{name}.{DESC_SUFFIX_NAME}"));
                    let mut desc = ExtentionObject::decode_from_file(output.as_path())?;

                    if let Some(messages) = subscribe_messages {
                        desc.mut_body().mut_content().set_subscribe_message_group(&messages);
                    }
                    if let Some(userdata) = userdata {
                        desc.mut_body().set_user_data(Some(userdata));
                    }
                    desc.mut_body().set_update_time();

                    desc
                }
            };

        ObjectDescOutput{
            output_path: Some(output),
            desc,
            desc_name: extention_name.to_owned(),
            private_key: None,
        }.output()

        // let core = match matches.value_of(core_service_arg_command.get_id()).map(| v | PathBuf::from(v)) {
        //     Some(core) => core,
        //     None => get_data_path(),
        // }.join(format!("{}.{DESC_SUFFIX_NAME}", "core-service"));
    
        // let core = if !core.exists() {
        //     return Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("{} isn't exist.", core.display())));
        // } else {
        //     DeviceObject::decode_from_file(core.as_path())?
        // };
    
        // let extention_name =
        //     matches.value_of(extention_name_arg_command.get_id())
        //         .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "extention name must exist."))?;
    
        // let subscribe_message = matches.value_of(subscribe_arg_command.get_id());
        // let output = matches.value_of(output_path_arg_command.get_id()).map(| v | PathBuf::from(v));
    
        // let o =
        //     ObjectBuilder::new(ExtentionDescContent::default(), ExtentionBodyContent::default())
        //         .update_desc(| desc |{
        //             desc.set_owner(Some(core.object_id().clone()));
        //             desc.mut_desc().set_extention_name(extention_name);
        //             if let Some(subscribe_message) = subscribe_message {
        //                 let messages: Vec<&str> = subscribe_message.split(' ').collect();
        //                 desc.mut_desc().subscribe_message_group(&messages);
        //             }
        //         })
        //         .update_body(| _body | {
        //         })
        //         .build()?;
    
        // output_desc(output, extention_name, o)?;
    
        // Ok(())
        
    }
}

struct ObjectDescOutput<DESC> {
    output_path: Option<PathBuf>,
    desc: DESC,
    desc_name: String,
    private_key: Option<PrivateKey>,
}

impl<DESC: FileEncoder> ObjectDescOutput<DESC> {

    pub fn output(self) -> NearResult<()> {

        let output = 
            if let Some(output) = self.output_path {
                if output.exists() {
                    output
                } else {
                    println!("{} not found, desc will save in ${{NEAR_HOME}}/data", output.display());
                    get_data_path()
                }
            } else {
                get_data_path()
            };

        let output_desc = | output: &PathBuf, name: &str, object: DESC | -> NearResult<()> {
            let output = output.join(format!("{name}.{DESC_SUFFIX_NAME}"));
            object.encode_to_file(output.as_path(), false)?;
            println!("output: {}", output.display());
            Ok(())
        };

        let output_prikey = | output: &PathBuf, name: &str, key: PrivateKey | -> NearResult<()> {
            key.encode_to_file(output.join(format!("{name}.{KEY_SUFFIX_NAME}")).as_path(), false)?;
            Ok(())
        };

        output_desc(&output, &self.desc_name, self.desc)?;

        if let Some(private_key) = self.private_key {
            output_prikey(&output, &self.desc_name, private_key)?;
        }

        Ok(())

    }

}
// impl DescToolBuilder<'_> {
//     pub fn build(self) -> NearResult<()> {

//         let owner = 
//             self.owner.map(| owner | {
//                 ObjectId::from_str(owner).expect("invalid owner")
//             });
//         let area = 
//             self.area.map(| area | Area::from_str(area).expect("invalid area"));
//         let output = self.output;
//         let userdata = self.userdata;
//         let pktype = 
//             self.pktype.map(| pktype | {
//                 let key: PkType = pktype.try_into().expect("invalid key");
//                 key.build().expect("failed gen key")
//             });
//         let host = 
//             self.host.map(| v | {
//                 std::net::Ipv4Addr::from_str(v).expect("invalid ip-addr")
//             });
//         let port_array = {
//             if let Some(port) = self.port {
//                 port.split(':').enumerate().into_iter().map(| (_, port) | port.parse::<u16>().expect("invalid ip-port")).collect()
//             } else {
//                 vec![CORE_STACK_PORT]
//             }
//         };
//             // self.port.map(| port | {
//             //     port.parse::<u16>().expect("invalid ip-port")
//             // })
//             // .unwrap_or(CORE_STACK_PORT);
//         let protocol = 
//             match self.protocol.unwrap_or("all") {
//                 "tcp" => enumflags2::make_bitflags!(Protocol::{ProtocolTcp}),
//                 "udp" => enumflags2::make_bitflags!(Protocol::{ProtocolUdp}),
//                 _ => enumflags2::make_bitflags!(Protocol::{ProtocolTcp | ProtocolUdp}),
//             };
//         let iptype = 
//             match self.network.unwrap_or("ipv4") {
//                 "ipv4" => NetworkType::Ipv4,
//                 "ipv6" => NetworkType::Ipv6,
//                 _ => NetworkType::IpAll,
//             };
//         let name = self.name;

//         let endpoints = {
//             let mut endpoints = vec![];

//             near_util::get_if_sockaddrs().expect("get ip address")
//                 .iter()
//                 .filter(| it | {
//                     match iptype {
//                         NetworkType::Ipv4 => it.is_ipv4(),
//                         NetworkType::Ipv6 => it.is_ipv6(),
//                         NetworkType::IpAll => true,
//                     }
//                 })
//                 .for_each(| it | {
//                     for &port in port_array.iter() {
//                         if it.is_ipv4() {
//                             if protocol.contains(Protocol::ProtocolTcp) {
//                                 endpoints.push(
//                                     if let Some(host) = host.as_ref() {
//                                         Endpoint::default_tcp(SocketAddr::new(IpAddr::from(host.clone()), port)).set_static_wan(true)
//                                     } else {
//                                         Endpoint::default_tcp(SocketAddr::new(it.clone(), port))
//                                     }
//                                 )
//                             }
//                             if protocol.contains(Protocol::ProtocolUdp) {
//                                 endpoints.push(
//                                     if let Some(host) = host.as_ref() {
//                                         Endpoint::default_udp(SocketAddr::new(IpAddr::from(host.clone()), port)).set_static_wan(true)
//                                     } else {
//                                         Endpoint::default_udp(SocketAddr::new(it.clone(), port))
//                                     }
//                                 )
//                             }
//                         } else {
//                             if protocol.contains(Protocol::ProtocolTcp) {
//                                 endpoints.push(Endpoint::default_tcp(SocketAddr::new(it.clone(), port)))
//                             }
//                             if protocol.contains(Protocol::ProtocolUdp) {
//                                 endpoints.push(Endpoint::default_udp(SocketAddr::new(it.clone(), port)))
//                             }
//                         }
//                     }
//                 });

//             endpoints
//         };

//         let (o, pktype) = 
//             match self.device_type {
//                 ServiceObjectType::Device(v) => {
//                     let pktype = pktype.expect("missing private key.");
//                     let o = 
//                         ObjectBuilder::new(DeviceDescContent::with_device(v),
//                                         DeviceBodyContent::default())
//                             .update_desc(| desc | {
//                                 desc.set_owner(owner);
//                                 desc.set_area(area);
//                                 desc.set_public_key(pktype.public());
//                             })
//                             .update_body(|body| {
//                                 body.mut_body().set_name(name);

//                                 if let Some(userdata) = userdata {
//                                     body.mut_body().set_userdata(userdata.into());
//                                 }

//                                 body.mut_body().set_endpoints(endpoints);
//                                 // body.mut_body().set_endpoints(endpoints)
//                             })
//                             .build()?;
//                     (o, pktype)
//                 }
//                 ServiceObjectType::Service(v) => {
//                     let pktype = pktype.expect("missing private key.");
//                     let o = 
//                         ObjectBuilder::new(DeviceDescContent::with_service(v),
//                                         DeviceBodyContent::default())
//                             .update_desc(| desc | {
//                                 desc.set_owner(owner);
//                                 desc.set_area(area);
//                                 desc.set_public_key(pktype.public());
//                             })
//                             .update_body(|body| {
//                                 body.mut_body().set_name(name);

//                                 if let Some(userdata) = userdata {
//                                     body.mut_body().set_userdata(userdata.into());
//                                 }

//                                 body.mut_body().set_endpoints(endpoints);
//                             })
//                             .build()?;
//                     (o, pktype)
//                 }
//             };

//         let name = 
//             name.map(| name | name.to_string())
//                 .unwrap_or(o.object_id().to_string());

//         output_prikey(output.clone(), name.as_str(), pktype)?;
//         output_desc(output, name.as_str(), o)?;

//         Ok(())

//     }
// }
