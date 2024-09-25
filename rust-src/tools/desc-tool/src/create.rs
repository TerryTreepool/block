
use std::{str::FromStr, path::PathBuf};

use clap::{ArgMatches, App, SubCommand};

use near_base::*;

use crate::h::*;

fn core_service_sub_command() -> App<'static> {
    SubCommand::with_name("core-service").about("Create core-service desc, it only one in NearOs.")
        .arg(owner_arg_command.clone())
        .arg(area_arg_command.clone())
        .arg(pktype_arg_command.clone())
        .arg(ipprotocol_arg_command.clone())
        .arg(iptype_arg_command.clone())
        .arg(ipaddr_arg_command.clone())
        .arg(ipport_arg_command.clone())
        .arg(output_path_arg_command.clone())
        .arg(userdata_arg_command.clone())
}

fn service_sub_command() -> App<'static> {
    SubCommand::with_name("service").about("Create service desc, it allow your service to run in Cluster services of NearOS.")
        .arg(area_arg_command.clone())
        .arg(pktype_arg_command.clone())
        .arg(ipprotocol_arg_command.clone())
        .arg(iptype_arg_command.clone())
        .arg(ipaddr_arg_command.clone())
        .arg(ipport_arg_command.clone())
        .arg(service_type_command.clone())
        .arg(output_path_arg_command.clone())
        .arg(userdata_arg_command.clone())
}

fn extention_sub_command() -> App<'static> {
    SubCommand::with_name("extention").about("Create extention desc, it allow your service to run in NearOS.")
        .arg(depended_desc_arg_command.clone())
        .arg(core_service_arg_command.clone())
        .arg(extention_name_arg_command.clone())
        .arg(subscribe_arg_command.clone())
        .arg(output_path_arg_command.clone())
}

fn people_sub_command() -> App<'static> {
    SubCommand::with_name("people").about("Create people desc, Allow identification of your identity in NearOS.")
        .arg(people_name_arg_command.clone())
        .arg(userdata_arg_command.clone())
        .arg(pktype_arg_command.clone())
        .arg(output_path_arg_command.clone())
}

pub fn create_subcommand() -> App<'static> {
    SubCommand::with_name("create").about("create desc")
        .subcommand(core_service_sub_command())
        .subcommand(service_sub_command())
        .subcommand(extention_sub_command())
        .subcommand(people_sub_command())
}

// fn gen_pktype<'a>(matches: &'a ArgMatches) -> NearResult<PrivateKey> {
//     let pktype = matches.value_of(pktype_arg_command.get_id()).map(| v | {
//         match v {
//             "rsa1024" => 1024,
//             "rsa2048" => 2048,
//             "rsa3072" | _ => 1024,
//         }
//     }).unwrap_or(1024);

//     match pktype {
//         1024 => PrivateKey::generate_rsa1024(),
//         2048 => PrivateKey::generate_rsa2048(),
//         _ => unreachable!()
//     }

// }

pub fn match_value<'a>(matches: &'a ArgMatches, id: &str) -> Option<&'a str> {
    if matches.try_get_raw(id).is_ok() {
        matches.value_of(id)
    } else {
        None
    }
}

fn create_service_desc<'a>(matches: &'a ArgMatches) -> NearResult<()> {
    let device_type = 
        ServiceObjectSubCode::from_str(
            match_value(matches, service_type_command.get_id()).expect("invalid service type")
        )
        .expect("invalid service type");
        
    DescToolBuilder {
        op: DescToolBuilderOp::Create,
        device_type: ServiceObjectType::Service(device_type as u8),
        owner: match_value(matches, owner_arg_command.get_id()),
        area: match_value(matches, area_arg_command.get_id()),
        pktype: match_value(matches, pktype_arg_command.get_id()),
        output: match_value(matches, output_path_arg_command.get_id()).map(| v | PathBuf::new().join(v)),
        userdata: match_value(matches, userdata_arg_command.get_id()),
        port: match_value(matches, ipport_arg_command.get_id()),
        host: match_value(matches, ipaddr_arg_command.get_id()),
        protocol: match_value(matches, ipprotocol_arg_command.get_id()),
        network: match_value(matches, iptype_arg_command.get_id()),
        name: Some({
            match device_type {
                ServiceObjectSubCode::OBJECT_TYPE_SERVICE_COTURN_MINER => "coturn-miner",
            }
        }),
        depended_desc: None,
        subscribe_messages: None,
    }
    .build()?
    .build()
}

fn create_core_service_desc<'a>(matches: &'a ArgMatches) -> NearResult<()> {
    DescToolBuilder {
        op: DescToolBuilderOp::Create,
        device_type: ServiceObjectType::Device(DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8),
        owner: match_value(matches, owner_arg_command.get_id()),
        area: match_value(matches, area_arg_command.get_id()),
        pktype: match_value(matches, pktype_arg_command.get_id()),
        output: match_value(matches, output_path_arg_command.get_id()).map(| v | PathBuf::new().join(v)),
        userdata: match_value(matches, userdata_arg_command.get_id()),
        port: match_value(matches, ipport_arg_command.get_id()),
        host: match_value(matches, ipaddr_arg_command.get_id()),
        protocol: match_value(matches, ipprotocol_arg_command.get_id()),
        network: match_value(matches, iptype_arg_command.get_id()),
        name: Some("core-service"),
        depended_desc: None,
        subscribe_messages: None,
    }
    .build()?
    .build()
}

pub fn create_extention_desc<'a>(matches: &'a ArgMatches) -> NearResult<()> {

    DescToolBuilder {
        op: DescToolBuilderOp::Create,
        device_type: ServiceObjectType::Extention,
        owner: None,
        area: None,
        pktype: None,
        output: match_value(matches, output_path_arg_command.get_id()).map(| v | PathBuf::from(v)),
        userdata: None,
        port: None,
        host: None,
        protocol: None,
        network: None,
        name: match_value(matches, extention_name_arg_command.get_id()),
        depended_desc: matches.value_of(depended_desc_arg_command.get_id()),
        subscribe_messages: matches.value_of(subscribe_arg_command.get_id()),
    }
    .build()?
    .build()

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

pub fn create_people_desc(matches: &ArgMatches) -> NearResult<()> {

    DescToolBuilder {
        op: DescToolBuilderOp::Create,
        device_type: ServiceObjectType::People,
        owner: None,
        area: None,
        pktype: match_value(matches, pktype_arg_command.get_id()),
        output: match_value(matches, output_path_arg_command.get_id()).map(| v | PathBuf::from(v)),
        userdata: match_value(matches, userdata_arg_command.get_id()),
        port: None,
        host: None,
        protocol: None,
        network: None,
        name: Some(match_value(matches, people_name_arg_command.get_id()).unwrap_or("BM")),
        depended_desc: None,
        subscribe_messages: None,
    }
    .build()?
    .build()
}

pub fn create_desc<'a>(matches: &'a ArgMatches) {
    if let Some(command) = matches.subcommand() {
        match command {
            ("core-service", matches) => {
                create_core_service_desc(matches).expect("failed gen core-service")
            }
            ("service", matches) => {
                create_service_desc(matches).expect("failed gen service")
            }
            ("extention", matches) => {
                create_extention_desc(matches).expect("failed gen extention")
            }
            ("people", matches) => {
                create_people_desc(matches).expect("failed gen people")
            }
            _ => {
            }
        }
    }
}