#![allow(non_upper_case_globals)]

use std::path::PathBuf;

use clap::{SubCommand, Arg, ArgMatches, App};
use near_base::{any::AnyNamedObject, FileDecoder, Serialize};

lazy_static::lazy_static! {
    static ref descfile_arg: Arg<'static> = 
        Arg::with_name("desc_file")
            .required(true)
            .index(1)
            .help("desc file to show");

    static ref show_owners_arg: Arg<'static> =
        Arg::with_name("show_owners")
            .long("owner")
            .help("show owner");

    static ref show_area_arg: Arg<'static> =
        Arg::with_name("show_area")
            .long("area")
            .help("show area");
    
    static ref show_public_arg: Arg<'static> =
        Arg::with_name("show_publickey")
            .long("key")
            .help("show public key");

    static ref show_author_arg: Arg<'static> = 
        Arg::with_name("show_author")
            .long("author")
            .help("show author");

    static ref show_all_arg: Arg<'static> = 
        Arg::with_name("show_all")
            .short('a')
            .long("all")
            .help("show all");

}

pub fn show_subcommand() -> App<'static> {
    SubCommand::with_name("show").about("show desc")
        .arg(descfile_arg.clone())
        .arg(show_owners_arg.clone())
        .arg(show_area_arg.clone())
        .arg(show_public_arg.clone())
        .arg(show_author_arg.clone())
        .arg(show_all_arg.clone())
}

const show_owners: u8       = 0b_0000_0001;
const show_area: u8         = 0b_0000_0010;
const show_publickey: u8    = 0b_0000_0100;
const show_author: u8       = 0b_0000_1000;
const show_all: u8          = show_owners | show_area | show_publickey | show_author;
pub fn show_desc(matches: &ArgMatches) {
    let mut show = 0u8;

    if matches.is_present(show_all_arg.get_id()) {
        show = show_all;
    } else {
        if matches.is_present(show_owners_arg.get_id()) {
            show |= show_owners;
        }
        if matches.is_present(show_area_arg.get_id()) {
            show |= show_area;
        }
        if matches.is_present(show_public_arg.get_id()) {
            show |= show_publickey;
        }
        if matches.is_present(show_author_arg.get_id()) {
            show |= show_author;
        }
    }

    let desc = matches.value_of(descfile_arg.get_id())
                                .map(| v | PathBuf::from(v) )
                                .unwrap();

    let desc = AnyNamedObject::decode_from_file(desc.as_path()).expect("parse desc");

    print!("object id: [");
    print!("{}", desc.object_id());
    print!("]");
    print!("\n");

    {
        let object_type_code_str = desc.object_type_code().to_string().expect("invalid code");
        print!("object type code: [");
        print!("{}", object_type_code_str);
        print!("]");
        print!("\n");
    }

    if show & show_owners == show_owners {
        print!("owner: [");
        if let Some(owner) = desc.owner() {
            print!("{}", owner);
        } else {
            print!("None");
        }
        print!("]");
        print!("\n");
    }

    if show & show_author == show_author {
        print!("author: [");
        if let Some(author) = desc.author() {
            print!("{}", author);
        } else {
            print!("None");
        }
        print!("]");
        print!("\n");
    }

    if show & show_area == show_area {
        print!("area: [");
        if let Some(area) = desc.area() {
            print!("{}", area);
        } else {
            print!("None");
        }
        print!("]");
        print!("\n");
    }

    if show & show_publickey == show_publickey {
        print!("pubkey: [");
        if let Some(pubkey) = desc.public_key() {
            let mut buf = vec![0u8; pubkey.raw_capacity()];
            pubkey.serialize(&mut buf).expect("build public key");
            print!("{}", hex::encode(buf));
        } else {
            print!("None");
        }
        print!("]");
        print!("\n");
    }

    if show  == show_all {
        match &desc {
            AnyNamedObject::Device(o) | 
            AnyNamedObject::Service(o) => {
                println!("body: [");
                println!("  endpoints:{{");
                for (idx, v) in o.body().content().endpoints().iter().enumerate() {
                    println!("    {}.{v}", idx+1);
                }
                print!("  }}\n");
                print!("]");
                print!("\n");
            }
            AnyNamedObject::Extention(o) => {
                println!("body: [");
                println!("  messages:{{");
                for (idx, message) in o.body().content().subscribe_messages().iter().enumerate() {
                    println!("    {}.{message}", idx+1);
                }
                print!("  }}\n");
                print!("]");
                print!("\n");
            }
            AnyNamedObject::People(o) => {
                println!("body: [");
                println!("  name:{{{}}}", o.body().content().name().unwrap_or(""));
                print!("]");
                print!("\n");
            }
            AnyNamedObject::Thing(thing) => {
                println!("body: [");
                println!("  mac-address:{{{}}}", hex::encode_upper(thing.desc().content().mac_address()));
                println!("  owner-depend-id:{{{}}}", thing.desc().content().owner_depend_id());
                println!("  thing-name:{{{}}}", thing.body().content().name());
                println!("  thing-data:{{");
                for (k, v) in thing.body().content().user_data() {
                    println!("    {k}={v}", );
                }
                print!("  }}\n");
                print!("]");
                print!("\n");
            }
            _ => {}
        }
    }

}
