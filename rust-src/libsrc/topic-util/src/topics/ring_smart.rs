
use near_util::{Topic, TopicBuilder, TopicStruct};

use crate::topic_types::TOPIC_P_CORE_LABEL;

pub const RING_LABEL: &'static str          = "ring";
pub const MAIN_CHAIN_LABEL: &'static str    = "main-chain";

lazy_static::lazy_static! {
    // publish
    static ref CORE_RING_CHAIN_PUBLISH: Topic =
        TopicBuilder::new(TOPIC_P_CORE_LABEL)
            .secondary(RING_LABEL)
            .add_thirdary(MAIN_CHAIN_LABEL)
            .add_thirdary("publish")
            .build();
    pub static ref CORE_RING_CHAIN_PUBLISH_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &CORE_RING_CHAIN_PUBLISH;
        TopicStruct::try_from(topic).unwrap()
    };

    // checkout
    static ref CORE_RING_CHAIN_CHECKOUT: Topic =
        TopicBuilder::new(TOPIC_P_CORE_LABEL)
            .secondary(RING_LABEL)
            .add_thirdary(MAIN_CHAIN_LABEL)
            .add_thirdary("checkout")
            .build();
    pub static ref CORE_RING_CHAIN_CHECKOUT_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &CORE_RING_CHAIN_CHECKOUT;
        TopicStruct::try_from(topic).unwrap()
    };

}
