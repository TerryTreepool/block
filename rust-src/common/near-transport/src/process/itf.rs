
use std::sync::Arc;

use near_base::{Serialize, Deserialize, };

pub trait ItfTrait: Send + Sync + Serialize + Deserialize + 'static {
}
pub type ItfTraitPtr = Arc<dyn ItfTrait>;

pub trait ItfBuilderTrait : Send + Sync {
    type R: Serialize + Deserialize;
    // fn build(&self, payload_max_len: usize) -> Vec<Self::R>;
    fn build(&self) -> Vec<Self::R>;
}
