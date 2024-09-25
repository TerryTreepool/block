
use near_util::{Topic, TopicBuilder};

/// It'a along self message in NearOS.
pub const TOPIC_P_NEAR_LABEL: &'static str    = "near";

/// It's along core mesage in NearOS.
pub const TOPIC_P_CORE_LABEL: &'static str    = "core";
/// It'a along system message in NearOS.
pub const TOPIC_P_SYSTEM_LABEL: &'static str  = "system";
/// It'a along kenerl message in NearOS.
pub const TOPIC_P_KENERL_LABEL: &'static str  = "kenerl";

pub const TOPIC_S_SUBSCRIBE_LABEL: &'static str = "subscribe";
pub const TOPIC_S_DISSUBSCRIBE_LABEL: &'static str = "dissubscribe";

lazy_static::lazy_static! {
    pub static ref TOPIC_SUBSCRIBE_STATIC: Topic = {
        TopicBuilder::new(TOPIC_P_CORE_LABEL)
            .secondary(TOPIC_S_SUBSCRIBE_LABEL)
            .build()
    };
}
