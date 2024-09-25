
use std::{collections::HashMap, sync::{Arc, RwLock}, ops::Add};

use mac_address::MacAddress;
use rlua::{AnyUserData, UserData, UserDataMethods};

#[derive(Default)]
struct DataImpl {
    cmd: String,
    mac: Option<MacAddress>,
    dataes: HashMap<String, String>,
}

#[derive(Clone)]
pub struct Data(Arc<RwLock<DataImpl>>);

impl std::default::Default for Data {
    fn default() -> Self {
        Self(Arc::new(
            Default::default()
        ))
    }
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (cmd, debug_string) = {
            let mut debug_string = String::default();

            let r = self.0.read().unwrap();

            for (k, v) in r.dataes.iter() {
                debug_string = debug_string.add(format!("thing_data[{k}]={v},").as_str());
            }

            (r.cmd.clone(), debug_string)
        };

        write!(f, "cmd: {cmd}, {debug_string}")
    }
}

impl From<HashMap<String, String>> for Data {
    fn from(value: HashMap<String, String>) -> Self {
        Self(Arc::new(RwLock::new(
            DataImpl { 
                dataes: value,
                ..Default::default()
            })))
    }
}

impl Data {
    pub fn take_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        std::mem::swap(&mut map, &mut self.0.write().unwrap().dataes);
        map
    }

    pub fn clone_map(&self) -> HashMap<String, String> {
        self.0.read().unwrap().dataes.clone()
    }

    pub fn set(&self, k: String, v: String) {
        let _ = 
        self.0
            .write().unwrap()
            .dataes
            .insert(k, v);
    }

    pub fn get(&self, k: &str) -> Option<String> {
        self.0
            .read().unwrap()
            .dataes
            .get(k)
            .cloned()
    }

    pub fn merge(&self, data: HashMap<String, String>) {
        self.0
            .write().unwrap()
            .dataes
            .extend(data);
    }

    pub fn set_cmd(&self, cmd: String) {
        self.0
            .write().unwrap()
            .cmd = cmd;
    }

    pub fn get_cmd(&self) -> String {
        self.0
            .read().unwrap()
            .cmd
            .clone()
    }

    pub fn set_mac(&self, mac: String) {
        if let Ok(mac) = mac.parse::<MacAddress>() {
            self.0
                .write().unwrap()
                .mac = Some(mac);
        }
    }

    pub fn get_mac(&self) -> Option<MacAddress> {
        self.0
            .read().unwrap()
            .mac
    }

}

impl UserData for Data {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("set", |_, (ud, k, v): (AnyUserData, String, String)| {
            // ud.borrow_mut::<Data>()?.set(String::from_utf8_lossy(k.as_slice()).to_string(), v);
            ud.borrow_mut::<Data>()?.set(k, v);
            Ok(())
        });

        methods.add_function("get", |_, (ud, k): (AnyUserData, String)| {
            // let r = ud.borrow::<Data>()?.get(String::from_utf8_lossy(k.as_slice()).to_string().as_str()).unwrap_or_default();
            let r = ud.borrow::<Data>()?.get(&k).unwrap_or_default();
            Ok(r)
        });

        methods.add_function("set_cmd", |_, (ud, cmd): (AnyUserData, String)| {
            // ud.borrow_mut::<Data>()?.set(String::from_utf8_lossy(k.as_slice()).to_string(), v);
            ud.borrow_mut::<Data>()?.set_cmd(cmd);
            Ok(())
        });

        methods.add_function("set_cmd", |_, (ud, cmd): (AnyUserData, String)| {
            // ud.borrow_mut::<Data>()?.set(String::from_utf8_lossy(k.as_slice()).to_string(), v);
            ud.borrow_mut::<Data>()?.set_cmd(cmd);
            Ok(())
        });

        methods.add_function("reset_mac", |_, (ud, mac): (AnyUserData, String)| {
            // ud.borrow_mut::<Data>()?.set(String::from_utf8_lossy(k.as_slice()).to_string(), v);
            ud.borrow_mut::<Data>()?.set_mac(mac);
            Ok(())
        });

    }
}
