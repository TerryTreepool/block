use std::path::PathBuf;

use near_base::*;
use near_core::*;
use protos::hci_service_config::Cfg;

macro_rules! my_macro {
    (struct $name:ident {
        $($field_name:ident: $field_type:ty,)*
    }) => {
        struct $name {
            $($field_name: $field_type,)*
        }

        impl $name {
            // This is purely an example—not a good one.
            pub fn get_field_names() -> Vec<&'static str> {
                vec![$(stringify!($field_name)),*]
            }

            pub fn (&mut self, name: &str, )
        }
    }
}

#[derive(Default)]
struct A {
    a: String,
    b: String,
    c: String,
}

pub fn load_config_with_string(cfg: &String) -> NearResult<String> {

}

pub fn load_config_with_table<T: protobuf::Message>(
    cfg: &mut toml::map::Map<String, toml::Value>
) -> NearResult<T> {
    let mut r = T::default();
    println!("{cfg}");

    let vals: Vec<(&String, &toml::Value)> = cfg.iter().collect();

    for (name, node) in vals {
        match node {
            toml::Value::Array(v) => { println!("array: {name}:{:?}", v); }
            toml::Value::Table(v) => { println!("table: {name}:{:?}", v); }
            toml::Value::Datetime(v) => { println!("datetime: {name}:{v}"); }
            toml::Value::String(v) => {
                println!("string: {name}:{v}");
                r
            }
            toml::Value::Boolean(v) => { println!("bool: {name}:{v}"); }
            toml::Value::Float(v) => { println!("float: {name}:{v}"); }
            toml::Value::Integer(v) => { println!("int: {name}:{v}"); }
        }
    }
    // for idx in cfg.as_array()
    // match cfg
    // unimplemented!()
    Ok(Default::default())
}

pub async fn load_from_config_v2<T: protobuf::Message>(service_name: &str) -> NearResult<T> {
    let toml_file = PathBuf::new().with_file_name(service_name).with_extension("toml");

    let content = async_std::fs::read_to_string(get_data_path().join(toml_file.as_path()))
        .await
        .map_err(|_| {
            let error_string = format!(
                "Missing [{}] file, will run with default configuration",
                toml_file.display()
            );
            println!("{error_string}");
            NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
        })?;

    let mut val: toml::Value = toml::from_str(&content).map_err(|e| {
        let error_string = format!("parse [{}] with err: {e}", toml_file.display());
        println!("{error_string}");
        NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, error_string)
    })?;

    let val = 
        val.as_table_mut()
            .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "cfg invalid."))?;

    load_config_with_table(val)
}

pub async fn load_from_config(service_name: &str) -> NearResult<()> {
    let toml_file = PathBuf::new()
        .with_file_name(service_name)
        .with_extension("toml");
    let content = async_std::fs::read_to_string(get_data_path().join(toml_file.as_path()))
        .await
        .map_err(|_| {
            let error_string = format!(
                "Missing [{}] file, will run with default configuration",
                toml_file.display()
            );
            println!("{error_string}");
            NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, error_string)
        })?;

    let mut val: toml::Value = toml::from_str(&content).map_err(|e| {
        let error_string = format!("parse [{}] with err: {e}", toml_file.display());
        println!("{error_string}");
        NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, error_string)
    })?;

    let load_routines = |val: &mut toml::Value| -> NearResult<()> {
        let routines = val
            .as_table_mut()
            .map(|table| table)
            .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not found [routines]."))?
            .remove("routines")
            .ok_or_else(|| {
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, "Not found [routines].")
            })?;

        let ctrl_interval = routines
            .get("ctrl_thing_task")
            .map(|cfg| {
                let ctrl_config = {
                    std::time::Duration::from_millis(
                        cfg.get("ctrl_interval")
                            .map(|ctrl_interval| ctrl_interval.as_integer().unwrap_or_default())
                            .unwrap_or_default() as u64,
                    )
                };
                ctrl_config
            })
            .unwrap_or(std::time::Duration::ZERO);

        println!("{:?}", ctrl_interval);

        Ok(())
    };

    load_routines(&mut val)?;

    Ok(())
}

#[async_std::main]
async fn main() {
    // load_from_config("hci-service").await.unwrap();

    // my_macro! {
    //     struct S {
    //         a: String,
    //         b: String,
    //     }
    // }

    // println!("{:?}", S::get_field_names());
    // println!("{:?}", stringify!(a));
    protos::protoc_utils::utils::init_protos_manager();

    let config: Cfg = load_from_config_v2("test").await.unwrap();

    println!("{config}");
}
