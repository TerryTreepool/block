
pub mod add_brand;
pub mod query_all_brand;
pub mod query_brand;
pub mod update_brand;

use near_base::hash_data;

pub(super) struct BrandIdBuilder<'a> {
    pub brand_name: &'a str,
}

impl BrandIdBuilder<'_> {
    pub(super) fn build(self) -> String {
        let binding = hash_data(&self.brand_name.as_bytes());
        let buf = binding.as_ref().as_ref();
        hex::encode_upper(&buf[0..16])
    }
}
