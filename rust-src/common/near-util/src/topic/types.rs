
use near_base::NearError;

use crate::{Topic, TopicRef};

pub(crate) const TOPIC_SEPARATOR: char = '/';

#[allow(unused)]
pub(crate) const PRIMARY_TOPIC_CORE_LABEL: &'static str    = "core";
#[allow(unused)]
pub(crate) const PRIMARY_TOPIC_SYSTEM_LABEL: &'static str  = "system";
#[allow(unused)]
pub(crate) const PRIMARY_TOPIC_NEAR_LABEL: &'static str    = "near";
#[allow(unused)]
pub(crate) const PRIMARY_TOPIC_KENERL_LABEL: &'static str  = "kenerl";

#[allow(unused)]
pub(crate) const SECONDARY_TOPIC_SUBSCRIBE_LABEL: &'static str = "subscribe";
#[allow(unused)]
pub(crate) const SECONDARY_TOPIC_DISSUBSCRIBE_LABEL: &'static str = "dissubscribe";

pub struct TopicStruct<'a> {
    topic: &'a Topic,
    topic_ref: TopicRef<'a>,
}

impl<'a> TryFrom<&'a Topic> for TopicStruct<'a> {
    type Error = NearError;
    fn try_from(topic: &'a Topic) -> Result<Self, Self::Error> {
        let topic_ref = topic.topic_d()?;
        Ok(Self{
            topic, 
            topic_ref,
        })
    }
}

impl TopicStruct<'_> {
    pub fn topic(&self) -> &Topic {
        self.topic
    }

    pub fn topic_ref(&self) -> &TopicRef {
        &self.topic_ref
    }
}

lazy_static::lazy_static! {
    /// subscribe
    static ref TOPIC_CORE_SUBSCRIBE_STATIC: Topic = super::build::Builder::new(PRIMARY_TOPIC_CORE_LABEL).secondary(SECONDARY_TOPIC_SUBSCRIBE_LABEL).build();
    pub static ref TOPIC_CORE_SUBSCRIBE: TopicStruct<'static> = {
        let topic: &'static Topic = &TOPIC_CORE_SUBSCRIBE_STATIC;
        TopicStruct::try_from(topic).unwrap()
    };

    /// dissubscribe
    static ref TOPIC_CORE_DISSUBSCRIBE_STATIC: Topic = super::build::Builder::new(PRIMARY_TOPIC_CORE_LABEL).secondary(SECONDARY_TOPIC_DISSUBSCRIBE_LABEL).build();
    pub static ref TOPIC_CORE_DISSUBSCRIBE: TopicStruct<'static> = {
        let topic: &'static Topic = &TOPIC_CORE_DISSUBSCRIBE_STATIC;
        TopicStruct::try_from(topic).unwrap()
    };
}
