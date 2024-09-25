
use near_base::{NearResult, NearError, ErrorCode};

use dataagent_util::Helper;
use protos::thing_group::{Thing_group_info, Thing_group_relation_info};

pub async fn get_group_list(db: &Helper) -> NearResult<Vec<Thing_group_info>> {

    db.query_all::<Thing_group_info>(crate::p::GET_ALL_GROUP.0)
        .await

}

pub async fn get_group(db: &Helper, group_id: &str) -> NearResult<Thing_group_info> {

    let mut group = 
    db.query_all_with_param::<Thing_group_info>(crate::p::GET_GROUP.0, 
                                        Thing_group_info {
                                            group_id: group_id.to_owned(),
                                            ..Default::default()
                                        })
        .await?
        .get_mut(0)
        .map(| item | {
            let mut group = Default::default();
            std::mem::swap(item, &mut group);
            group
        })
        .ok_or_else(|| {
            NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found [{}] group.", group_id))
        })?;

    group.set_thing_relation(
        {
            let info = 
                db.query_all_with_param::<Thing_group_relation_info>(
                    crate::p::QUERY_ALL_THING_GROUP_RELATION.0, 
                    Thing_group_relation_info {
                        group_id: group_id.to_owned(),
                        ..Default::default()
                    })
                .await?;

            info.into_iter()
                .map(| item | {
                    thing_data_property::decode(item)
                })
                .collect()
        }
    );

    Ok(group)
    
}



pub async fn get_group_relation(db: &Helper, 
                                group_id: &str, 
                                thing_id: &str) -> NearResult<Thing_group_relation_info> {
    let relation = 
      db.query_all_with_param::<Thing_group_relation_info>(
            crate::p::QUERY_THING_GROUP_RELATION.0, 
            Thing_group_relation_info {
                group_id: group_id.to_owned(),
                thing_id: thing_id.to_owned(),
                ..Default::default()
            })
        .await?
        .get_mut(0)
        .map(| item | {
            let mut relation = Default::default();
            std::mem::swap(&mut relation, item);
            relation
        })
        .ok_or_else(|| {
            NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("[{}-{}] isnot exist.", group_id, thing_id))
        })?;

    Ok(thing_data_property::decode(relation))

}

pub(crate) mod thing_data_property {
    use std::collections::HashMap;

    use log::warn;
    use near_base::{Deserialize, Serialize};
    use protos::thing_group::Thing_group_relation_info;

    pub(crate) fn decode(mut data: Thing_group_relation_info) -> Thing_group_relation_info {
        let text = data.take_thing_data_property_text();

        data.set_thing_data_property({
            hex::decode(text.as_str())
                .map(| bytes | {
                    HashMap::<String, String>::deserialize(bytes.as_slice())
                        .map(| (v, _) | {
                            v
                        })
                        .map_err(| e | {
                            warn!("[{}-{} relation property invalid data, exception-data: {}]", data.group_id(), data.thing_id(), text);
                            e
                        })
                        .unwrap_or(Default::default())
                })
                .map_err(| e | {
                    warn!("[{}-{} relation property invalid hex-data, exception-data: {}.]", data.group_id(), data.thing_id(), text);
                    e
                })
                .unwrap_or(Default::default())
        });

        data
    }

    pub(crate) fn encode(mut data: Thing_group_relation_info) -> Thing_group_relation_info {
        data.set_thing_data_property_text({
            let mut v = vec![0u8; data.thing_data_property.raw_capacity()];
            let _ = data.thing_data_property.serialize(&mut v).unwrap();
            hex::encode_upper(v)
        });

        data
    }
}