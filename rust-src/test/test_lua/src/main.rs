
use std::{io::Read, path::PathBuf};

use rlua::Function;

// use crate::configure::ConfigureData;

// mod configure;
mod lua;

fn main() {
    let params: Vec<String> = std::env::args().skip(1).collect();

    if params.len() == 0 {
        panic!("lua file is none.");
    }

    let (lua_function, lua_files) = params.split_last().unwrap();

    let manager = crate::lua::manager::Manager::new();

    for f in lua_files {
        manager.load(PathBuf::new().join(f)).expect("failed load");
    }

    let v = manager.call("test_lua_2", &lua_function).expect("");

    println!("{}", hex::encode_upper(v));

    // let lua_file = params.get(0).unwrap();
    // println!("parse lua file: {}", lua_file);

    // let lua_function = params.get(1).expect("missing function");
    // println!("exec {lua_function}");

    // let data = {
    //     let mut fs = 
    //         std::fs::OpenOptions::new()
    //             .read(true)
    //             .open(lua_file)
    //             .expect("failed read");
    //     let mut data = vec![0u8; fs.metadata().unwrap().len() as usize];
    //     fs.read_exact(data.as_mut_slice());
    //     data
    // };

    // let lua = rlua::Lua::new();

    // // load lua file
    // lua.context(| ctx | {
    //     ctx.load(data.as_slice())
    //         .exec()
    //         .expect("failed load lua");

    //     let globals = ctx.globals();
    //     let userdata = ctx.create_userdata(ConfigureData::get_instace().clone()).unwrap();
    //     globals.set("configure_data", userdata).unwrap();
    // });

    // let mut v = vec![0u8; 31];
    // let v_ptr = v.as_mut_ptr();
    // lua.context(|ctx| {
    //     let globals = ctx.globals();
    //     let fun = globals.get::<_, Function>(lua_function.as_str()).unwrap();

    //     let v = fun.call::<Vec<u8>, Vec<u8>>(v).unwrap();

    //     println!("{}", hex::encode_upper(v));

    // });

}
