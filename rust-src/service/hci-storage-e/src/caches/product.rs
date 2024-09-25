
use near_base::{Serialize, hash_data, Deserialize, NearResult, ErrorCode, NearError};

use protos::hci::product::*;
use storage::ItemTrait;

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
        let buf = binding.as_slice();
        // let buf = binding.as_ref().as_ref();
        hex::encode_upper(&buf[0..16])
    }
}

#[derive(Clone)]
pub struct ProductItem {
    product: Product_info,
}

impl ProductItem {
    pub fn create_new(product_name: String) -> NearResult<Self> {
        let product_name = product_name.as_str().trim();
        if product_name.is_empty() {
            Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "product name can't empty."))
        } else {
            Ok(())
        }?;

        Ok(
            Self {
                product: Product_info { 
                    product_id: ProductIdBuilder {
                        parent_product_id: "",
                        product_name: &product_name,
                    }.build(),
                    parent_product_id: Default::default(),
                    product_name: product_name.to_owned(),
                    ..Default::default()
                },
            }
        )
    }

    pub fn insert_child(&mut self, child_product_name: String) -> bool {

        match ProductItem::create_new(child_product_name) {
            Ok(child_product) => {
                let insert = self.children().products().iter().find(| &product | product.product_id() == child_product.product_id()).is_none();
                if insert {
                    self.product.mut_children().mut_products().push(child_product.product);
                }

                insert
            }
            _ => { false }
        }

    }

    pub fn remove_child(&mut self, child_product_id: &str) {
        let children = self.product.take_children();

        let children = 
            children.products.into_iter()
                .filter(| product_id | {
                    !product_id.product_id().eq(child_product_id)
                })
                .collect();

        self.product.set_children(Product_info_list {
            products: children,
            ..Default::default()
        });
    }

    pub fn take(self) -> Product_info {
        self.product
    }

}

impl ItemTrait for ProductItem {
    fn id(&self) -> &str {
        self.product.product_id()
    }
}

impl Serialize for ProductItem {
    fn raw_capacity(&self) -> usize {
        self.product.raw_capacity()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> near_base::NearResult<&'a mut [u8]> {
        let buf = self.product.serialize(buf)?;

        Ok(buf)
    }
}

impl Deserialize for ProductItem {
    fn deserialize<'de>(buf: &'de [u8]) -> near_base::NearResult<(Self, &'de [u8])> {
        let (product_info, buf) = Product_info::deserialize(buf)?;

        Ok((Self {
            product: product_info,
        }, buf))
    }
}

impl std::ops::Deref for ProductItem {
    type Target = Product_info;

    fn deref(&self) -> &Self::Target {
        &self.product
    }
}
