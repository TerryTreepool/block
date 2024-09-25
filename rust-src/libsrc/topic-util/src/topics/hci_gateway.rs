use near_util::{Topic, TopicBuilder, TopicStruct};

use crate::topic_types::TOPIC_P_NEAR_LABEL;

pub const THING_LABEL: &'static str     = "thing";
const THING_GATEWAY_LABEL: &'static str = "gateway";

lazy_static::lazy_static! {
    // query all device
    static ref NEAR_THING_GATEWAY_QUERY_ALL_THING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(THING_GATEWAY_LABEL)
            .add_thirdary("query-all-thing")
            .build();
    pub static ref NEAR_THING_GATEWAY_QUERY_ALL_THING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_GATEWAY_QUERY_ALL_THING;
        TopicStruct::try_from(topic).unwrap()
    };


    // search device
    static ref NEAR_THING_GATEWAY_SEARCH: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(THING_GATEWAY_LABEL)
            .add_thirdary("search-thing")
            .build();
    pub static ref NEAR_THING_GATEWAY_SEARCH_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_GATEWAY_SEARCH;
        TopicStruct::try_from(topic).unwrap()
    };

    // search device result
    static ref NEAR_THING_GATEWAY_SEARCH_RESULT: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(THING_GATEWAY_LABEL)
            .add_thirdary("search-result")
            .build();
    pub static ref NEAR_THING_GATEWAY_SEARCH_RESULT_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_GATEWAY_SEARCH_RESULT;
        TopicStruct::try_from(topic).unwrap()
    };

    // add thing
    static ref NEAR_THING_GATEWAY_ADD_THING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(THING_GATEWAY_LABEL)
            .add_thirdary("add-thing")
            .build();
    pub static ref NEAR_THING_GATEWAY_ADD_THING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_GATEWAY_ADD_THING;
        TopicStruct::try_from(topic).unwrap()
    };

    // crud thing
    static ref NEAR_THING_GATEWAY_CRUD_THING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(THING_GATEWAY_LABEL)
            .add_thirdary("crud-thing")
            .build();
    pub static ref NEAR_THING_GATEWAY_CRUD_THING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_GATEWAY_CRUD_THING;
        TopicStruct::try_from(topic).unwrap()
    };

    // ctrl thing
    static ref NEAR_THING_GATEWAY_CTRL_THING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(THING_GATEWAY_LABEL)
            .add_thirdary("ctrl-thing")
            .build();
    pub static ref NEAR_THING_GATEWAY_CTRL_THING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_GATEWAY_CTRL_THING;
        TopicStruct::try_from(topic).unwrap()
    };

    // schedule
    // add
    static ref NEAR_THING_GATEWAY_SCHEDULE_ADD: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(THING_GATEWAY_LABEL)
            .add_thirdary("schedule")
            .add_thirdary("add")
            .build();
    pub static ref NEAR_THING_GATEWAY_SCHEDULE_ADD_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_GATEWAY_SCHEDULE_ADD;
        TopicStruct::try_from(topic).unwrap()
    };

    // remove
    static ref NEAR_THING_GATEWAY_SCHEDULE_REMOVE: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(THING_GATEWAY_LABEL)
            .add_thirdary("schedule")
            .add_thirdary("remove")
            .build();
    pub static ref NEAR_THING_GATEWAY_SCHEDULE_REMOVE_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_GATEWAY_SCHEDULE_REMOVE;
        TopicStruct::try_from(topic).unwrap()
    };

    // update
    static ref NEAR_THING_GATEWAY_SCHEDULE_UPDATE: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(THING_GATEWAY_LABEL)
            .add_thirdary("schedule")
            .add_thirdary("update")
            .build();
    pub static ref NEAR_THING_GATEWAY_SCHEDULE_UPDATE_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_GATEWAY_SCHEDULE_UPDATE;
        TopicStruct::try_from(topic).unwrap()
    };

    // update relations
    static ref NEAR_THING_GATEWAY_SCHEDULE_UPDATE_RELATIONS: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(THING_GATEWAY_LABEL)
            .add_thirdary("schedule")
            .add_thirdary("update-relations")
            .build();
    pub static ref NEAR_THING_GATEWAY_SCHEDULE_UPDATE_RELATIONS_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_GATEWAY_SCHEDULE_UPDATE_RELATIONS;
        TopicStruct::try_from(topic).unwrap()
    };
}
