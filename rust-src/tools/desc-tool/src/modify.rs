
use std::{path::PathBuf, str::FromStr};

use clap::{App, SubCommand, ArgMatches};

use near_base::*;

use crate::{create::match_value, h::*};

fn modify_core_service_desc<'a>(matches: &'a ArgMatches) -> NearResult<()> {
    DescToolBuilder {
        op: DescToolBuilderOp::Modify,
        device_type: ServiceObjectType::Device(DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8),
        owner: None,
        area: None,
        pktype: None,
        output: match_value(matches, output_path_arg_command.get_id()).map(| v | PathBuf::from(v)),
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

fn modify_service_desc<'a>(matches: &'a ArgMatches) -> NearResult<()> {
    let device_type = 
        ServiceObjectSubCode::from_str(
            match_value(matches, service_type_command.get_id()).expect("invalid service type")
        )
        .expect("invalid service type");
        
    DescToolBuilder {
        op: DescToolBuilderOp::Modify,
        device_type: ServiceObjectType::Service(device_type as u8),
        owner: None,
        area: None,
        pktype: None,
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

fn modify_extention_desc<'a>(matches: &'a ArgMatches) -> NearResult<()> {
    DescToolBuilder {
        op: DescToolBuilderOp::Modify,
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
        depended_desc: match_value(matches, depended_desc_arg_command.get_id()),
        subscribe_messages: match_value(matches, subscribe_arg_command.get_id()),
    }
    .build()?
    .build()
}

fn core_service_sub_command() -> App<'static> {
    SubCommand::with_name("core-service").about("Modify core-service desc, it only one in NearOs.")
        .arg(ipprotocol_arg_command.clone())
        .arg(iptype_arg_command.clone())
        .arg(ipaddr_arg_command.clone())
        .arg(ipport_arg_command.clone())
        .arg(output_path_arg_command.clone())
        .arg(userdata_arg_command.clone())
}

fn service_sub_command() -> App<'static> {
    SubCommand::with_name("service").about("Modify service desc, it allow your service to run in Cluster services of NearOS.")
        .arg(service_type_command.clone())
        .arg(ipprotocol_arg_command.clone())
        .arg(iptype_arg_command.clone())
        .arg(ipaddr_arg_command.clone())
        .arg(ipport_arg_command.clone())
        .arg(output_path_arg_command.clone())
        .arg(userdata_arg_command.clone())
}

fn extention_sub_command() -> App<'static> {
    SubCommand::with_name("extention").about("Modify extention desc, it allow your service to run in NearOS.")
        .arg(extention_name_arg_command.clone())
        .arg(subscribe_arg_command.clone())
        .arg(output_path_arg_command.clone())
}

pub fn modify_subcommand() -> App<'static> {
    SubCommand::with_name("modify").about("modify desc")
        .subcommand(core_service_sub_command())
        .subcommand(extention_sub_command())
        .subcommand(service_sub_command())
}

pub fn modify_desc<'a>(matches: &'a ArgMatches) {
    if let Some(command) = matches.subcommand() {
        match command {
            ("core-service", matches) => {
                modify_core_service_desc(matches).expect("failed modify core-service")
            }
            ("service", matches) => {
                modify_service_desc(matches).expect("failed modify service")
            }
            ("extention", matches) => {
                modify_extention_desc(matches).expect("failed modify extention")
            }
            // ("people", matches) => {
            //     modify_people_desc(matches).expect("failed modify people")
            // }
            _ => {
                panic!("don't support modify {}", command.0);
            }
        }
    }
}