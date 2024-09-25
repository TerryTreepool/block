
use std::collections::HashMap;

use near_base::{NearResult,
                ObjectId, Area, thing::{ThingObject, ThingDescContent, ThingBodyContent}, ObjectBuilder,
    };

pub struct ThingBuilder<'a> {
    owner: Option<&'a ObjectId>,
    author: Option<&'a ObjectId>,
    area: Option<Area>,
    mac_address: Option<[u8; 6]>,
    owner_depend_id: String,
    _name: String,
    user_data: Option<HashMap<String, String>>,
}

impl<'a> ThingBuilder<'a> {
    pub fn new() -> Self {
        Self {
            owner: None,
            author: None,
            area: None,
            mac_address: None,
            owner_depend_id: Default::default(),
            _name: Default::default(),
            user_data: None,
        }
    }

    pub fn owner(mut self, owner: Option<&'a ObjectId>) -> Self {
        self.owner = owner;
        self
    }

    pub fn author(mut self, author: Option<&'a ObjectId>) -> Self {
        self.author = author;
        self
    }

    pub fn area(mut self, area: Option<Area>) -> Self {
        self.area = area;
        self
    }

    pub fn mac_address(mut self, mac_address: [u8; 6]) -> Self {
        self.mac_address = Some(mac_address);
        self
    }

    pub fn owner_depend_id(mut self, owner_depend_id: String) -> Self {
        self.owner_depend_id = owner_depend_id;
        self
    }

    pub fn user_data(mut self, user_data: HashMap<String, String>) -> Self {
        self.user_data = Some(user_data);
        self
    }
}

impl ThingBuilder<'_> {
    pub fn build(self) -> NearResult<ThingObject> {
        let thing = 
            ObjectBuilder::new(ThingDescContent::new(),
                                ThingBodyContent::default())
                .update_desc(| desc | {
                    desc.no_create_time();
                    desc.set_owner(self.owner.cloned());
                    desc.set_author(self.author.cloned());
                    desc.set_area(self.area);
                    if let Some(mac) = self.mac_address {
                        desc.mut_desc().set_mac_address(mac);
                    }

                    desc.mut_desc().set_owner_depend_id(self.owner_depend_id);

                })
                .update_body(| body | {
                    if let Some(userdata) = self.user_data {
                        body.mut_body().set_userdata(userdata);
                    }
                })
                .build()?;

        Ok(thing)
    }
}


#[test]
#[allow(unused)]
fn test_thing_build() {
    use std::path::PathBuf;
    use near_base::FileEncoder;

    let mut map = HashMap::new();
    map.insert("A".to_owned(), "01".to_owned());
    map.insert("B".to_owned(), "02".to_owned());
    let thing_build = ThingBuilder {
        owner: None,
        author: None,
        area: None,
        owner_depend_id: "1235".to_owned(),
        mac_address: Some([0x11, 0x11, 0x11, 0x11, 0x7A, 0x0B]),
        _name: "Thing".to_owned(),
        user_data: Some(map),
    }
    .build()
    .unwrap();

    println!("object-id: {}", thing_build.object_id());
    // let _ = thing_build.encode_to_file(PathBuf::new().join("c:").with_file_name("1").with_extension("desc").as_path(), false).unwrap();

}
