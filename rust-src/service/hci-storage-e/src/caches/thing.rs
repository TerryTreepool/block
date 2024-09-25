
use std::{collections::HashMap, str::FromStr};

use common::RuntimeStack;
use mac_address::MacAddress;
use near_base::{NearResult, Serialize, Deserialize, thing::ThingObject, NearError, ErrorCode};

use near_util::ThingBuilder;
use storage::ItemTrait;
use protos::hci::thing::*;

pub struct ThingItemBuild<'a> {
    pub(crate) brand_id: &'a str,
    pub(crate) major_product_id: &'a str,
    pub(crate) minor_product_id: &'a str,
    pub(crate) thing_name: &'a str,
    pub(crate) thing_mac: &'a str,
    pub(crate) thing_data: HashMap<String, String>,
}

impl ThingItemBuild<'_> {
    pub fn build(self) -> NearResult<ThingItem> {

        let mut thing = Thing_info {
            brand_id: self.brand_id.to_owned(),
            major_product_id: self.major_product_id.to_owned(),
            minor_product_id: self.minor_product_id.to_owned(),
            thing_id: Default::default(),
            thing_name: self.thing_name.to_owned(),
            mac_address: self.thing_mac.to_string(),
            ..Default::default()
        };

        let mac = 
            MacAddress::from_str(self.thing_mac)
                .map_err(| e | {
                    let error_string = format!("failed parse to mac-address with err: {e}");
                    NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, error_string)
                })?;

        let thing_object = 
            ThingBuilder::new()
                .owner(Some(RuntimeStack::get_instance().stack().core_device().object_id()))
                .mac_address(mac.bytes())
                .owner_depend_id(self.brand_id.to_owned())
                .user_data(self.thing_data)
                .build()?;

        thing.set_thing_id(thing_object.object_id().to_string());

        Ok(ThingItem { 
            thing, 
            thing_object, 
        })

    }
}

#[derive(Clone)]
pub struct ThingItem {
    thing: Thing_info,
    thing_object: ThingObject,
}

impl ThingItem {
    pub fn thing(&self) -> &Thing_info {
        &self.thing
    }

    pub fn set_thing_name(&mut self, thing_name: String) {
        self.thing.set_thing_name(thing_name);
    }

    pub fn split(self) -> (Thing_info, ThingObject) {
        (self.thing, self.thing_object)
    }
}

impl ItemTrait for ThingItem {
    fn id(&self) -> &str {
        self.thing.thing_id()
    }
}

impl Serialize for ThingItem {
    fn raw_capacity(&self) -> usize {
        self.thing_object.raw_capacity() +
        self.thing.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        let buf = self.thing_object.serialize(buf)?;
        let buf = self.thing.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for ThingItem {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (thing_object, buf) = ThingObject::deserialize(buf)?;
        let (thing, buf) = Thing_info::deserialize(buf)?;

        Ok((Self {
            thing,
            thing_object,
        }, buf))
    }
}
