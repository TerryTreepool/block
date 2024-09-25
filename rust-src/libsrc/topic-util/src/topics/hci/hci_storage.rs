

use near_util::{Topic, TopicBuilder, TopicStruct};

use crate::topic_types::TOPIC_P_NEAR_LABEL;

const PUB_LABEL: &'static str       = "pub";
const BRAND_LABEL: &'static str     = "brand";
const PRODUCT_LABEL: &'static str   = "product";
const THING_LABEL: &'static str     = "thing";
const SCHEDULE_LABEL: &'static str  = "schedule";
const STORAGE_LABEL: &'static str   = "storage";

// transactino
lazy_static::lazy_static! {
    // begin transaction
    static ref NEAR_THING_STORAGE_PUB_BEGIN: Topic = 
    TopicBuilder::new(TOPIC_P_NEAR_LABEL)
        .secondary(THING_LABEL)
        .add_thirdary(STORAGE_LABEL)
        .add_thirdary(PUB_LABEL)
        .add_thirdary("begin")
        .build();
    pub static ref NEAR_THING_STORAGE_PUB_BEGIN_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_PUB_BEGIN;
        TopicStruct::try_from(topic).unwrap()
    };

    // commit
    static ref NEAR_THING_STORAGE_PUB_COMMIT: Topic = 
    TopicBuilder::new(TOPIC_P_NEAR_LABEL)
        .secondary(THING_LABEL)
        .add_thirdary(STORAGE_LABEL)
        .add_thirdary(PUB_LABEL)
        .add_thirdary("commit")
        .build();
    pub static ref NEAR_THING_STORAGE_PUB_COMMIT_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_PUB_COMMIT;
        TopicStruct::try_from(topic).unwrap()
    };

    // rollback
    static ref NEAR_THING_STORAGE_PUB_ROLLBACK: Topic = 
    TopicBuilder::new(TOPIC_P_NEAR_LABEL)
        .secondary(THING_LABEL)
        .add_thirdary(STORAGE_LABEL)
        .add_thirdary(PUB_LABEL)
        .add_thirdary("rollback")
        .build();
    pub static ref NEAR_THING_STORAGE_PUB_ROLLBACK_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_PUB_ROLLBACK;
        TopicStruct::try_from(topic).unwrap()
    };
}

// brand
lazy_static::lazy_static! {
    // query all brand
    static ref NEAR_THING_STORAGE_BRAND_QUERY_ALL: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(BRAND_LABEL)
            .add_thirdary("query-all")
            .build();
    pub static ref NEAR_THING_STORAGE_BRAND_QUERY_ALL_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_BRAND_QUERY_ALL;
        TopicStruct::try_from(topic).unwrap()
    };

    // query brand
    static ref NEAR_THING_STORAGE_BRAND_QUERY: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(BRAND_LABEL)
            .add_thirdary("query")
            .build();
    pub static ref NEAR_THING_STORAGE_BRAND_QUERY_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_BRAND_QUERY;
        TopicStruct::try_from(topic).unwrap()
    };

    // add brand
    static ref NEAR_THING_STORAGE_BRAND_ADD: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(BRAND_LABEL)
            .add_thirdary("add")
            .build();
    pub static ref NEAR_THING_STORAGE_BRAND_ADD_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_BRAND_ADD;
        TopicStruct::try_from(topic).unwrap()
    };

    // update brand
    static ref NEAR_THING_STORAGE_BRAND_REMOVE: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(BRAND_LABEL)
            .add_thirdary("update")
            .build();
    pub static ref NEAR_THING_STORAGE_BRAND_REMOVE_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_BRAND_REMOVE;
        TopicStruct::try_from(topic).unwrap()
    };
}

// product
lazy_static::lazy_static!{
    // add
    static ref NEAR_THING_STORAGE_PRODUCT_ADD: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(PRODUCT_LABEL)
            .add_thirdary("add")
            .build();
    pub static ref NEAR_THING_STORAGE_PRODUCT_ADD_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_PRODUCT_ADD;
        TopicStruct::try_from(topic).unwrap()
    };

    // update
    static ref NEAR_THING_STORAGE_PRODUCT_REMOVE: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(PRODUCT_LABEL)
            .add_thirdary("update")
            .build();
    pub static ref NEAR_THING_STORAGE_PRODUCT_REMOVE_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_PRODUCT_REMOVE;
        TopicStruct::try_from(topic).unwrap()
    };

    // query
    static ref NEAR_THING_STORAGE_PRODUCT_QUERY: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(PRODUCT_LABEL)
            .add_thirdary("query")
            .build();
    pub static ref NEAR_THING_STORAGE_PRODUCT_QUERY_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_PRODUCT_QUERY;
        TopicStruct::try_from(topic).unwrap()
    };

    // query all
    static ref NEAR_THING_STORAGE_PRODUCT_QUERY_ALL: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(PRODUCT_LABEL)
            .add_thirdary("query-all")
            .build();
    pub static ref NEAR_THING_STORAGE_PRODUCT_QUERY_ALL_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_PRODUCT_QUERY_ALL;
        TopicStruct::try_from(topic).unwrap()
    };
}

// device
lazy_static::lazy_static! {

    // add
    static ref NEAR_THING_STORAGE_THING_ADD: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(THING_LABEL)
            .add_thirdary("add")
            .build();
    pub static ref NEAR_THING_STORAGE_THING_ADD_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_THING_ADD;
        TopicStruct::try_from(topic).unwrap()
    };

    // update
    static ref NEAR_THING_STORAGE_THING_UPDATE: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(THING_LABEL)
            .add_thirdary("update")
            .build();
    pub static ref NEAR_THING_STORAGE_THING_UPDATE_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_THING_UPDATE;
        TopicStruct::try_from(topic).unwrap()
    };

    // remove
    static ref NEAR_THING_STORAGE_THING_REMOVE: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(THING_LABEL)
            .add_thirdary("remove")
            .build();
    pub static ref NEAR_THING_STORAGE_THING_REMOVE_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_THING_REMOVE;
        TopicStruct::try_from(topic).unwrap()
    };

    // query all
    static ref NEAR_THING_STORAGE_THING_QUERY_ALL: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(THING_LABEL)
            .add_thirdary("query-all")
            .build();
    pub static ref NEAR_THING_STORAGE_THING_QUERY_ALL_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_THING_QUERY_ALL;
        TopicStruct::try_from(topic).unwrap()
    };

    // query
    static ref NEAR_THING_STORAGE_THINGOBJECT_QUERY: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(THING_LABEL)
            .add_thirdary("query")
            .build();
    pub static ref NEAR_THING_STORAGE_THINGOBJECT_QUERY_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_THINGOBJECT_QUERY;
        TopicStruct::try_from(topic).unwrap()
    };

    // query multiple
    static ref NEAR_THING_STORAGE_MULITPLE_THINGOBJECT_QUERY: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(THING_LABEL)
            .add_thirdary("query-multiple-thing")
            .build();
    pub static ref NEAR_THING_STORAGE_MULITPLE_THINGOBJECT_QUERY_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_MULITPLE_THINGOBJECT_QUERY;
        TopicStruct::try_from(topic).unwrap()
    };

}

// thing-group
lazy_static::lazy_static! {

    // add
    static ref NEAR_THING_STORAGE_SCHEDULE_ADD: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(SCHEDULE_LABEL)
            .add_thirdary("add")
            .build();
    pub static ref NEAR_THING_STORAGE_SCHEDULE_ADD_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_SCHEDULE_ADD;
        TopicStruct::try_from(topic).unwrap()
    };

    // update
    static ref NEAR_THING_STORAGE_SCHEDULE_UPDATE: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(SCHEDULE_LABEL)
            .add_thirdary("update")
            .build();
    pub static ref NEAR_THING_STORAGE_SCHEDULE_UPDATE_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_SCHEDULE_UPDATE;
        TopicStruct::try_from(topic).unwrap()
    };

    // remove
    static ref NEAR_THING_STORAGE_SCHEDULE_REMOVE: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(SCHEDULE_LABEL)
            .add_thirdary("remove")
            .build();
    pub static ref NEAR_THING_STORAGE_SCHEDULE_REMOVE_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_SCHEDULE_REMOVE;
        TopicStruct::try_from(topic).unwrap()
    };

    // query
    static ref NEAR_THING_STORAGE_SCHEDULE_QUERY: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(SCHEDULE_LABEL)
            .add_thirdary("query")
            .build();
    pub static ref NEAR_THING_STORAGE_SCHEDULE_QUERY_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_SCHEDULE_QUERY;
        TopicStruct::try_from(topic).unwrap()
    };

    // query-all
    static ref NEAR_THING_STORAGE_SCHEDULE_QUERYALL: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(SCHEDULE_LABEL)
            .add_thirdary("query-all")
            .build();
    pub static ref NEAR_THING_STORAGE_SCHEDULE_QUERYALL_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_SCHEDULE_QUERYALL;
        TopicStruct::try_from(topic).unwrap()
    };

    // insert thing
    static ref NEAR_THING_STORAGE_SCHEDULE_INSERT_THING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(SCHEDULE_LABEL)
            .add_thirdary("insert-thing")
            .build();
    pub static ref NEAR_THING_STORAGE_SCHEDULE_INSERT_THING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_SCHEDULE_INSERT_THING;
        TopicStruct::try_from(topic).unwrap()
    };

    // remove thing
    static ref NEAR_THING_STORAGE_SCHEDULE_REMOVE_THING: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(SCHEDULE_LABEL)
            .add_thirdary("remove-thing")
            .build();
    pub static ref NEAR_THING_STORAGE_SCHEDULE_REMOVE_THING_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_SCHEDULE_REMOVE_THING;
        TopicStruct::try_from(topic).unwrap()
    };

    // update relation
    static ref NEAR_THING_STORAGE_SCHEDULE_RELATIONS_UPDATE: Topic = 
        TopicBuilder::new(TOPIC_P_NEAR_LABEL)
            .secondary(THING_LABEL)
            .add_thirdary(STORAGE_LABEL)
            .add_thirdary(SCHEDULE_LABEL)
            .add_thirdary("relations")
            .add_thirdary("update")
            .build();
    pub static ref NEAR_THING_STORAGE_SCHEDULE_RELATIONS_UPDATE_PUB: TopicStruct<'static> = {
        let topic: &'static Topic = &NEAR_THING_STORAGE_SCHEDULE_RELATIONS_UPDATE;
        TopicStruct::try_from(topic).unwrap()
    };

}
