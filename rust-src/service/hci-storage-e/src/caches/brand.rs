
use near_base::{NearResult, Serialize, Deserialize, NearError, ErrorCode, hash_data};

use protos::hci::brand::*;
use storage::ItemTrait;

struct BrandIdBuilder<'a> {
    brand_name: &'a str,
}

impl BrandIdBuilder<'_> {
    fn build(self) -> String {
        let binding = hash_data(&self.brand_name.as_bytes());
        let buf = binding.as_slice();
        // let buf = binding.as_ref().as_ref();
        hex::encode_upper(&buf[0..16])
    }
}

#[derive(Clone)]
pub struct BrandItem {
    brand_info: Brand_info
}

impl BrandItem {
    pub fn create_new(brand_name: String) -> NearResult<Self> {
        let brand_name = brand_name.as_str().trim();
        if brand_name.is_empty() {
            Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "brand name can't empty."))
        } else {
            Ok(())
        }?;

        Ok(
            Self {
                brand_info: Brand_info { 
                    brand_id: BrandIdBuilder{ brand_name: &brand_name }.build(), 
                    brand_name: brand_name.to_owned(),
                    ..Default::default()
                }
            }
        )
    }

}

impl From<BrandItem> for Brand_info {
    fn from(value: BrandItem) -> Self {
        value.brand_info
    }
}

impl From<Brand_info> for BrandItem {
    fn from(value: Brand_info) -> Self {
        Self{
            brand_info: value,
        }
    }
}

impl ItemTrait for BrandItem {
    fn id(&self) -> &str {
        self.brand_id()
    }
}

impl Serialize for BrandItem {
    fn raw_capacity(&self) -> usize {
        self.brand_info.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        self.brand_info.serialize(buf)
    }
}

impl Deserialize for BrandItem {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        let (brand, buf) = Brand_info::deserialize(buf)?;

        Ok((brand.into(), buf))
    }
}

impl std::ops::Deref for BrandItem {
    type Target = Brand_info;

    fn deref(&self) -> &Self::Target {
        &self.brand_info
    }
}
