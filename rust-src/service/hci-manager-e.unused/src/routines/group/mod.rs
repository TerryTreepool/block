
pub mod add;
pub mod update;
pub mod query;
pub mod query_all;
pub mod insert_thing;
pub mod remove_thing;

mod p;

use near_base::{Serialize, hash_data};

pub(super) struct GroupIdBuilder<'a> {
    pub group_name: &'a str,
    pub _now: near_base::Timestamp,
}

impl GroupIdBuilder<'_> {
    pub(super) fn build(self) -> String {
        let buf = {
            let mut buf = vec![0u8; self.group_name.raw_capacity()];

            let _end = self.group_name.serialize(&mut buf).unwrap();
            // let _ = self.now.serialize(end).unwrap();
            buf
        };

        let binding = hash_data(buf.as_slice());
        let buf = binding.as_ref().as_ref();
        hex::encode_upper(&buf[0..16])
    }
}
