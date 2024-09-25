
use near_util::{Topic, TopicBuilder, TopicStruct};

use crate::topic_types::TOPIC_P_CORE_LABEL;

pub const SN_SMART_LABEL: &'static str  = "sn";
pub const SMART_LABEL: &'static str     = "smart";

lazy_static::lazy_static! {
    // ping
    static ref CORE_SN_SMART_PING: Topic =
        TopicBuilder::new(TOPIC_P_CORE_LABEL)
            .secondary(SN_SMART_LABEL)
            .add_thirdary(SMART_LABEL)
            .add_thirdary("ping")
            .build();
    pub static ref CORE_SN_SMART_PING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &CORE_SN_SMART_PING;
        TopicStruct::try_from(topic).unwrap()
    };

    // check-out
    static ref CORE_SN_SMART_CHECKOUT: Topic =
        TopicBuilder::new(TOPIC_P_CORE_LABEL)
            .secondary(SN_SMART_LABEL)
            .add_thirdary(SMART_LABEL)
            .add_thirdary("check-out")
            .build();
    pub static ref CORE_SN_SMART_CHECKOUT_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &CORE_SN_SMART_CHECKOUT;
        TopicStruct::try_from(topic).unwrap()
    };

    // call
    static ref CORE_SN_SMART_CALL: Topic =
        TopicBuilder::new(TOPIC_P_CORE_LABEL)
            .secondary(SN_SMART_LABEL)
            .add_thirdary(SMART_LABEL)
            .add_thirdary("call")
            .build();
    pub static ref CORE_SN_SMART_CALL_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &CORE_SN_SMART_CALL;
        TopicStruct::try_from(topic).unwrap()
    };

    // invite
    static ref CORE_SN_SMART_INVITE: Topic =
        TopicBuilder::new(TOPIC_P_CORE_LABEL)
            .secondary(SN_SMART_LABEL)
            .add_thirdary(SMART_LABEL)
            .add_thirdary("invite")
            .build();
    pub static ref CORE_SN_SMART_INVITE_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &CORE_SN_SMART_INVITE;
        TopicStruct::try_from(topic).unwrap()
    };

}
