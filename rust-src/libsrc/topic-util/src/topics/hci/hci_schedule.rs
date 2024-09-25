
use near_util::{TopicBuilder, Topic, TopicStruct};

use crate::{topic_types::TOPIC_P_NEAR_LABEL, topics::hci_gateway::THING_LABEL};

const SERVICE_LABEL: &'static str   = "schedule";

lazy_static::lazy_static!{
    // sync schedule
    static ref NEAR_THING_SCHEDULE_SYNCTHING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("sync-thing")
            .build();
    pub static ref NEAR_THING_SCHEDULE_SYNCTHING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SCHEDULE_SYNCTHING;
        TopicStruct::try_from(topic).unwrap()
    };

    // remove thing schedule
    static ref NEAR_THING_SCHEDULE_REMOVETHING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("remove-thing")
            .build();
    pub static ref NEAR_THING_SCHEDULE_REMOVETHING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SCHEDULE_REMOVETHING;
        TopicStruct::try_from(topic).unwrap()
    };

    // add schedule
    static ref NEAR_THING_SCHEDULE_ADD: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("add")
            .build();
    pub static ref NEAR_THING_SCHEDULE_ADD_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SCHEDULE_ADD;
        TopicStruct::try_from(topic).unwrap()
    };

    // remove schedule
    static ref NEAR_THING_SCHEDULE_REMOVE: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("remove")
            .build();
    pub static ref NEAR_THING_SCHEDULE_REMOVE_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SCHEDULE_REMOVE;
        TopicStruct::try_from(topic).unwrap()
    };

    // execute schedule
    static ref NEAR_THING_SCHEDULE_EXECUTE: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("execute")
            .build();
    pub static ref NEAR_THING_SCHEDULE_EXECUTE_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SCHEDULE_EXECUTE;
        TopicStruct::try_from(topic).unwrap()
    };
}
