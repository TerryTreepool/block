
use near_util::{TopicBuilder, Topic, TopicStruct};

use crate::{topic_types::TOPIC_P_NEAR_LABEL, topics::hci_gateway::THING_LABEL};

const SERVICE_LABEL: &'static str   = "service";

lazy_static::lazy_static!{
    // search devices by brand
    static ref NEAR_THING_SERVICE_SEARCH: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("search")
            .build();
    pub static ref NEAR_THING_SERVICE_SEARCH_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SERVICE_SEARCH;
        TopicStruct::try_from(topic).unwrap()
    };

    // search result
    static ref NEAR_THING_SERVICE_TASK_RESULT: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("task-result")
            .build();
    pub static ref NEAR_THING_SERVICE_TASK_RESULT_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SERVICE_TASK_RESULT;
        TopicStruct::try_from(topic).unwrap()
    };

    // add device
    static ref NEAR_THING_SERVICE_ADD_THING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("add-thing")
            .build();
    pub static ref NEAR_THING_SERVICE_ADD_THING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SERVICE_ADD_THING;
        TopicStruct::try_from(topic).unwrap()
    };

    // remove device
    static ref NEAR_THING_SERVICE_REMOVE_THING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("remove-thing")
            .build();
    pub static ref NEAR_THING_SERVICE_REMOVE_THING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SERVICE_REMOVE_THING;
        TopicStruct::try_from(topic).unwrap()
    };

    // pair thing
    static ref NEAR_THING_SERVICE_PAIR_THING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("pair-thing")
            .build();
    pub static ref NEAR_THING_SERVICE_PAIR_THING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SERVICE_PAIR_THING;
        TopicStruct::try_from(topic).unwrap()
    };

    // remove pair thing
    static ref NEAR_THING_SERVICE_REMOVE_PAIR_THING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("remove-pair-thing")
            .build();
    pub static ref NEAR_THING_SERVICE_REMOVE_PAIR_THING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SERVICE_REMOVE_PAIR_THING;
        TopicStruct::try_from(topic).unwrap()
    };

    // query thing
    static ref NEAR_THING_SERVICE_QUERY_THING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("query-thing")
            .build();
    pub static ref NEAR_THING_SERVICE_QUERY_THING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SERVICE_QUERY_THING;
        TopicStruct::try_from(topic).unwrap()
    };

    // control thing
    static ref NEAR_THING_SERVICE_CONTROL_THING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("ctrl-thing")
            .build();
    pub static ref NEAR_THING_SERVICE_CONTROL_THING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SERVICE_CONTROL_THING;
        TopicStruct::try_from(topic).unwrap()
    };

    // control things
    static ref NEAR_THING_SERVICE_CONTROL_THING_ARRAY: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("ctrl-things")
            .build();
    pub static ref NEAR_THING_SERVICE_CONTROL_THING_ARRAY_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SERVICE_CONTROL_THING_ARRAY;
        TopicStruct::try_from(topic).unwrap()
    };

    // query all thing
    static ref NEAR_THING_SERVICE_QUERY_ALL_THING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("query-all-thing")
            .build();
    pub static ref NEAR_THING_SERVICE_QUERY_ALL_THING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SERVICE_QUERY_ALL_THING;
        TopicStruct::try_from(topic).unwrap()
    };

    // add schedule
    static ref NEAR_THING_SERVICE_SCHEDULE_ADD: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("schedule")
            .add_thirdary("add")
            .build();
    pub static ref NEAR_THING_SERVICE_SCHEDULE_ADD_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SERVICE_SCHEDULE_ADD;
        TopicStruct::try_from(topic).unwrap()
    };

    // remove schedule
    static ref NEAR_THING_SERVICE_SCHEDULE_REMOVE: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("schedule")
            .add_thirdary("remove")
            .build();
    pub static ref NEAR_THING_SERVICE_SCHEDULE_REMOVE_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SERVICE_SCHEDULE_REMOVE;
        TopicStruct::try_from(topic).unwrap()
    };

    // execute schedule
    static ref NEAR_THING_SERVICE_SCHEDULE_EXECUTE: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(SERVICE_LABEL)
            .add_thirdary("schedule")
            .add_thirdary("execute")
            .build();
    pub static ref NEAR_THING_SERVICE_SCHEDULE_EXECUTE_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_SERVICE_SCHEDULE_EXECUTE;
        TopicStruct::try_from(topic).unwrap()
    };

}
