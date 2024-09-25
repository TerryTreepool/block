
use common::RoutineTemplate;
use log::{error, info};
use near_base::builder_codec_utils::Empty;
use protos::hci_schedule::{Schedule_data, Hci_schedule_add, hci_schedule_add::Schedule_type};
use topic_util::{topics::hci_schedule::{NEAR_THING_SCHEDULE_ADD_PUB, NEAR_THING_SCHEDULE_REMOVE_PUB}, types::brand_types::Status};

use crate::process::Process;

pub(crate) fn sync_group_schedule(process: Process, group_id: String) {
    async_std::task::spawn(async move {
        let mut group = 
            crate::public::group::get_group(process.db_helper(), &group_id)
                .await
                .map_err(| e | {
                    error!("{e}");
                    e
                })?;

        let status = 
            Status::try_from(group.status())
                .map_err(| e | {
                    error!("{e}");
                    e
                })?;

        match status {
            Status::Eanbled => {
                let schedule_data: Vec<Schedule_data> = 
                group.take_thing_relation()
                    .into_iter()
                    .map(| mut data | {
                        Schedule_data {
                            thing_id: data.take_thing_id(),
                            thing_data: data.take_thing_data_property(),
                            ..Default::default()
                        }
                    })
                    .collect();
    
            RoutineTemplate::<Empty>::call(
                    NEAR_THING_SCHEDULE_ADD_PUB.topic().clone(), 
                    Hci_schedule_add {
                        schedule_id: group.take_group_id(),
                        // @@protoc_insertion_point(field:hci_schedule_add.m)
                        m: Schedule_type::Manual.into(),
                        // @@protoc_insertion_point(field:hci_schedule_add.thing)
                        thing: schedule_data,
                        ..Default::default()
                    }
                )
                .map_err(| e | {
                    error!("{e}");
                    e
                })?
                .await
                .map(| _ | {
                    info!("Successfully sync [{}] to hci-schedule", group)
                })
                .map_err(| e | {
                    error!("{e}");
                    e
                })
            }
            Status::Disabled => {
                RoutineTemplate::<Empty>::call(
                    NEAR_THING_SCHEDULE_REMOVE_PUB.topic().clone(), 
                        group.take_group_id()
                )
                .map_err(| e | {
                    error!("{e}");
                    e
                })?
                .await
                .map(| _ | {
                    info!("Successfully remove [{}] to hci-schedule", group)
                })
                .map_err(| e | {
                    error!("{e}");
                    e
                })
            }
        }
    });
}