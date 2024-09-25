
use near_base::{Serialize, hash_data};

pub mod add_product;
pub mod query;
pub mod query_all;
pub mod update;

pub(super) struct ProductIdBuilder<'a> {
    pub parent_product_id: &'a str,
    pub product_name: &'a str,
}

impl ProductIdBuilder<'_> {
    pub(super) fn build(self) -> String {
        let buf = {
            let mut buf = vec![0u8; self.parent_product_id.raw_capacity() + self.product_name.raw_capacity()];

            let end = self.parent_product_id.serialize(&mut buf).unwrap();
            let _ = self.product_name.serialize(end).unwrap();
            buf
        };

        let binding = hash_data(buf.as_slice());
        let buf = binding.as_ref().as_ref();
        hex::encode_upper(&buf[0..16])
    }
}
