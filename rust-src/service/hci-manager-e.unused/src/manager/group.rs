
use std::{process::Output, pin::Pin};

use futures::Future;
use near_base::{NearResult, NearError, ErrorCode};

use protos::thing_group::{Thing_group_info, Thing_group_relation_info};
use topic_util::types::brand_types::Status;

use super::manager_template::{ItemTrait, UpdateItemTrait, CheckTrait, UpdateItemTrait_V2, UpdateItemTrait_V2_CB};

#[derive(Clone)]
pub struct GroupItem {
    group: Thing_group_info,
}

impl GroupItem {
    // pub fn insert_thing(&mut self, thing_ids: impl Iterator<Item=String>) {
    //     for thing_id in thing_ids {
    //         if !self.thing_ids().contains(&thing_id) {
    //             self.mut_thing_ids().push(thing_id);
    //         }
    //     }
    // }

    // pub fn remove_thing(&mut self, thing_ids: &[String]) {
    //     let things = 
    //         self.take_thing_ids()
    //             .into_iter()
    //             .filter(| thing_id | {
    //                 !thing_ids.contains(thing_id)
    //             })
    //             .collect();

    //     self.set_thing_ids(things);
    // }
}

impl From<GroupItem> for Thing_group_info {
    fn from(value: GroupItem) -> Self {
        value.group
    }
}

impl From<Thing_group_info> for GroupItem {
    fn from(value: Thing_group_info) -> Self {
        Self {
            group: value,
        }
    }
}

impl ItemTrait for GroupItem {
    fn get_item_id(&self) -> &str {
        self.group.group_id()
    }
}

impl UpdateItemTrait<GroupItem> for GroupItem {
    fn update_item(&mut self, new_item: GroupItem) {
        let mut_group = &mut self.group;
        let group: Thing_group_info = new_item.into();

        debug_assert_eq!(mut_group.group_id(), group.group_id());

        mut_group.set_status(group.status);
        mut_group.set_update_time(group.update_time);
    }
}

// impl UpdateItemTrait_V2<GroupItem, Vec<String>> for GroupItem {
//     fn update_item<CB: Fn(&Brand_info)>(&mut self, new_item: BrandItem, cb: CB) -> NearResult<BrandItem> {
//         let mut_brand_info = &mut self.brand_info;

//         debug_assert_eq!(mut_brand_info.brand_id(), new_item.brand_info.brand_id());

//         mut_brand_info.set_update_time(new_item.brand_info.update_time);
//         mut_brand_info.set_status(new_item.brand_info.status);

//         cb(mut_brand_info);

//         Ok(mut_brand_info.clone().into())
//     }
// }

// pub trait UpdateItemTrait_V2<O, P> { 
//     fn update_item<CB: Fn(&P)>(&mut self, context: O, cb: CB) -> NearResult<()>;

impl<R> UpdateItemTrait_V2<Thing_group_relation_info, Thing_group_relation_info, R> for GroupItem
where R: Future<Output = NearResult<()>> {
    fn insert_item<CB>(&mut self, context: Thing_group_relation_info, mut cb: CB) -> NearResult<R>
    where   CB: UpdateItemTrait_V2_CB<Thing_group_relation_info, R> {

        if let Some(_) = self.thing_relation()
                             .iter()
                             .find(| &item | {
                                 item.thing_id() == context.thing_id()
                             }) {
            Err(NearError::new(ErrorCode::NEAR_ERROR_ALREADY_EXIST, format!("[{}] has been exist.", context.thing_id())))
        } else {
            Ok(())
        }?;

        self.mut_thing_relation().push(context.clone());

        let r = cb.call(context);

        r
    }

    fn update_item<CB>(&mut self, mut context: Thing_group_relation_info, mut cb: CB) -> NearResult<R>
    where   CB: UpdateItemTrait_V2_CB<Thing_group_relation_info, R>
    {

        if let Some(item) = 
            self.mut_thing_relation()
                .iter_mut()
                .find(| item | {
                    item.thing_id() == context.thing_id()
                }) {
            item.set_thing_data_property(context.take_thing_data_property());

            cb.call(item.clone())
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_DONOT_EXIST, format!("[{}] isnot exist.", context.thing_id())))
        }
    }
}



impl CheckTrait for GroupItem {
    fn check_status(&self) -> NearResult<()> {
        self.status.try_into()
            .map_or_else(| _ |{
                let error_string = format!("{}'s status is exception, can't use it.", self.group_id());
                Err(NearError::new(ErrorCode::NEAR_ERROR_EXCEPTION, error_string))
            },| status | {
                match status {
                    Status::Eanbled => Ok(()),
                    Status::Disabled => {
                        let error_string = format!("{} status is diabled, cann't use it.", self.group_id());
                        Err(NearError::new(ErrorCode::NEAR_ERROR_NO_AVAILABLE, error_string))
                    }
                }
            })
    }
}

impl std::ops::Deref for GroupItem {
    type Target = Thing_group_info;

    fn deref(&self) -> &Self::Target {
        &self.group
    }
}

impl std::ops::DerefMut for GroupItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.group
    }
}