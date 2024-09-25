
use std::{sync::{Arc, RwLock}, collections::{BTreeMap, btree_map::Entry}, str::FromStr};

use chrono::{Timelike, Datelike};
use common::RoutineTemplate;
use enumflags2::{bitflags, BitFlags, make_bitflags};

use near_base::{NearResult, ErrorCode, NearError, builder_codec_macro::Empty};

use log::{warn, trace, error, info};
use protos::hci::schedule::{Schedule_info, Schedule_mode, schedule_timeperiod_mode::Schedule_cycle_week};
use topic_util::{types::thing_data::{ThingId, ThingData}, topics::hci_service::{NEAR_THING_SERVICE_SCHEDULE_EXECUTE_PUB, NEAR_THING_SERVICE_QUERY_ALL_THING_PUB}};

use super::{ScheduleTrait, OnSchedultEventTrait, schedule_data::ScheduleData, ScheduleTraitRef, schedule_cycletime::CycleTimeComponents};

#[derive(Clone)]
enum Components {
    Manual(ScheduleTraitRef<ManualComponents>),
    TimePeriod(ScheduleTraitRef<TimePeriodComponents>),
    Condition(ScheduleTraitRef<ConditionComponents>),
}

impl From<ManualComponents> for Components {
    fn from(v: ManualComponents) -> Self {
        Self::Manual(ScheduleTraitRef::new(v))
    }
}

impl From<TimePeriodComponents> for Components {
    fn from(v: TimePeriodComponents) -> Self {
        Self::TimePeriod(ScheduleTraitRef::new(v))
    }
}

impl From<ConditionComponents> for Components {
    fn from(v: ConditionComponents) -> Self {
        Self::Condition(ScheduleTraitRef::new(v))
    }
}

impl Components {
    pub fn manual_components(&self) -> Option<ScheduleTraitRef<ManualComponents>> {
        match self {
            Self::Manual(c) => Some(c.clone()),
            _ => None
        }
    }

    pub fn timeperiod_componentes(&self) -> Option<ScheduleTraitRef<TimePeriodComponents>> {
        match self {
            Self::TimePeriod(c) => Some(c.clone()),
            _ => None
        }
    }

    #[allow(unused)]
    pub fn condition_componentes(&self) -> Option<ScheduleTraitRef<ConditionComponents>> {
        match self {
            Self::Condition(c) => Some(c.clone()),
            _ => None
        }
    }

}

pub(super) struct ManualComponents {
    schedule_id: String,
    schedule_data: ScheduleData,
}

impl ManualComponents {
    pub fn new(schedule_id: &str) -> Self {
        Self { 
            schedule_id: schedule_id.to_owned(),
            schedule_data: ScheduleData::default(),
        }
    }

    #[inline]
    pub fn schedule_id(&self) -> &str {
        self.schedule_id.as_str()
    }
}

#[async_trait::async_trait]
impl ScheduleTrait<Vec<(ThingId, ThingData)>> for ManualComponents {
    fn update_schedule(&self, things: Vec<(ThingId, ThingData)>) {
        self.schedule_data.update_schedule(things);
    }

    fn remove_schedule(&self, things: Vec<ThingId>) {
        self.schedule_data.remove_schedule(things);
    }

    async fn execute<E: OnSchedultEventTrait>(&self, event: E) -> near_base::NearResult<()> {
        self.schedule_data.execute(event).await
    }

    async fn release(&self) {

    }
}

#[bitflags]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
enum CycleWeek {
    Mon = 1 << 0,
    Tues = 1 << 1,
    Wed = 1 << 2,
    Thur = 1 << 3,
    Fri = 1 << 4,
    Sat = 1 << 5,
    Sun = 1 << 6,
}

impl From<chrono::Weekday> for CycleWeek {
    fn from(value: chrono::Weekday) -> Self {
        match value {
            chrono::Weekday::Mon => Self::Mon,
            chrono::Weekday::Tue => Self::Tues,
            chrono::Weekday::Wed => Self::Wed,
            chrono::Weekday::Thu => Self::Thur,
            chrono::Weekday::Fri => Self::Fri,
            chrono::Weekday::Sat => Self::Sat,
            chrono::Weekday::Sun => Self::Sun,
        }
    }
}

#[derive(Default, Clone)]
struct CycleWeekTime {
    cycle_time: chrono::NaiveTime,
    flags: BitFlags<CycleWeek>,
}

#[derive(Default, Clone)]
enum CycleState {
    #[default]
    CycleNone,
    CycleWeek(CycleWeekTime),
    CycleOnce(chrono::NaiveTime),
}

struct TimePeriodComponents {
    schedule_id: String,
    cycle_state: RwLock<CycleState>,
    schedule_data: ScheduleData,
}

impl TimePeriodComponents {
    pub fn new(schedule_id: &str) -> Self {
        Self {
            schedule_id: schedule_id.to_owned(),
            cycle_state: RwLock::new(Default::default()),
            schedule_data: Default::default(),
        }
    }

    #[inline]
    pub fn schedule_id(&self) -> &str {
        self.schedule_id.as_str()
    }

}

#[async_trait::async_trait]
impl ScheduleTrait<(CycleState, Vec<(ThingId, ThingData)>)> for TimePeriodComponents {
    fn update_schedule(&self, context: (CycleState, Vec<(ThingId, ThingData)>)) {
        let (mut cycle_state, things) = context;

        {
            let mut_cycle_state = &mut *self.cycle_state.write().unwrap();

            std::mem::swap(mut_cycle_state, &mut cycle_state);
        }

        self.schedule_data.update_schedule(things);
    }

    fn remove_schedule(&self, things: Vec<ThingId>) {
        self.schedule_data.remove_schedule(things);
    }

    async fn execute<E: OnSchedultEventTrait>(&self, event: E) -> near_base::NearResult<()> {
        let (today, now_time) = {
            let now = chrono::Local::now();

            (
                CycleWeek::from(now.weekday()),
                chrono::NaiveTime::from_hms_opt(now.hour(), now.minute(), 0)
                    .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_FATAL, "Unable to obtain the current time"))?
            )
        };

        let cur_cycle_state = {
            self.cycle_state.read().unwrap().clone()
        };

        let execute_flag = 
            match &cur_cycle_state {
                CycleState::CycleOnce(cycle_time) => {
                    now_time == *cycle_time
                }
                CycleState::CycleWeek(cycle_week) => {
                    if cycle_week.flags.contains(today) {
                        now_time == cycle_week.cycle_time
                    } else {
                        false
                    }
                }
                _ => { false /* Ingore */ }
            };

        if execute_flag {
            match &cur_cycle_state {
                CycleState::CycleOnce(_) => *self.cycle_state.write().unwrap() = CycleState::CycleNone,
                _ => { /* ignore */ }
            }

            self.schedule_data.execute(event).await
        } else {
            Ok(())
        }
    }

    async fn release(&self) {
        
    }
}

struct ConditionComponents {
    _schedule_id: String,
}

impl ConditionComponents {

    #[inline]
    #[allow(unused)]
    pub fn schedule_id(&self) -> &str {
        self._schedule_id.as_str()
    }

}

#[async_trait::async_trait]
impl ScheduleTrait<Vec<(ThingId, ThingData)>> for ConditionComponents {
    fn update_schedule(&self, _things: Vec<(ThingId, ThingData)>) {
    }

    fn remove_schedule(&self, _things: Vec<ThingId>) {
    }

    async fn execute<E: OnSchedultEventTrait>(&self, _event: E) -> near_base::NearResult<()> {
        Ok(())
    }

    async fn release(&self) {
        
    }
}

struct ManagerImpl {
    components: RwLock<BTreeMap<String, Components>>,

    fix_cycle_time_component: ScheduleTraitRef<CycleTimeComponents>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn new() -> Self {
        Self(Arc::new(ManagerImpl {
            components: RwLock::new(BTreeMap::new()),
            fix_cycle_time_component: ScheduleTraitRef::new(CycleTimeComponents::new(std::time::Duration::from_secs(60))),
        }))
    }

    pub fn update_schedule(&self, mut schedule_info: Schedule_info) -> NearResult<()> {
        let m = 
            schedule_info.mode.enum_value()
                .map_err(| e | {
                    NearError::new(ErrorCode::NEAR_ERROR_UNDEFINED, format!("Undefined [{}] schedule type.", e))
                })?;

        let mut thing_dataes = vec![];

        for mut data in schedule_info.take_thing_relation().into_iter() {
            if let Ok(thing_id) = ThingId::from_str(data.thing_id()) {
                thing_dataes.push((thing_id, ThingData::from(data.take_thing_data_property())));
            } else {
                warn!("warning: [{}] thing-id is invalid.", data.thing_id());
            }
        }

        match m {
            Schedule_mode::Maual => self.update_manual_schedule(schedule_info.take_schedule_id(), thing_dataes),
            Schedule_mode::TimePeriod => {
                let cycle_state = 
                    if schedule_info.has_timeperiod_mode() {
                        let timeperiod_mode = schedule_info.take_timeperiod_mode();

                        let cycle_time = 
                            if timeperiod_mode.has_time() {
                                chrono::NaiveTime::from_hms_opt(timeperiod_mode.time().hour, timeperiod_mode.time().minute, 0)
                                    .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT, format!("{{{}:{}}} isn't time format.", timeperiod_mode.time().hour, timeperiod_mode.time().minute)))
                            } else {
                                Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "cycle time can't empty."))
                            }?;

                        let mut cycle_week_flags = BitFlags::empty();
                        let cycle_week = timeperiod_mode.cycle();

                        if (cycle_week & Schedule_cycle_week::Mon  as u32) > 0 { cycle_week_flags.extend(make_bitflags!(CycleWeek::{Mon}))}
                        if (cycle_week & Schedule_cycle_week::Tues as u32) > 0 { cycle_week_flags.extend(make_bitflags!(CycleWeek::{Tues}))}
                        if (cycle_week & Schedule_cycle_week::Wed  as u32) > 0 { cycle_week_flags.extend(make_bitflags!(CycleWeek::{Wed}))}
                        if (cycle_week & Schedule_cycle_week::Thur as u32) > 0 { cycle_week_flags.extend(make_bitflags!(CycleWeek::{Thur}))}
                        if (cycle_week & Schedule_cycle_week::Fri  as u32) > 0 { cycle_week_flags.extend(make_bitflags!(CycleWeek::{Fri}))}
                        if (cycle_week & Schedule_cycle_week::Sat  as u32) > 0 { cycle_week_flags.extend(make_bitflags!(CycleWeek::{Sat}))}
                        if (cycle_week & Schedule_cycle_week::Sun  as u32) > 0 { cycle_week_flags.extend(make_bitflags!(CycleWeek::{Sun}))}

                        if cycle_week_flags.is_empty() {
                            CycleState::CycleOnce(cycle_time)
                        } else {
                            CycleState::CycleWeek(
                                CycleWeekTime {
                                    cycle_time,
                                    flags: cycle_week_flags
                                }
                            )
                        }

                    } else {
                        CycleState::CycleNone
                    };

                self.update_timeperiod_schedule(schedule_info.take_schedule_id(), 
                                                cycle_state,
                                                thing_dataes)
            }
            _ => { Ok(()) }
        }
    }

    pub(in self) fn update_manual_schedule(&self, 
                                  schedule_id: String, 
                                  things: Vec<(ThingId, ThingData)>) -> NearResult<()> {
        let component = {
            match self.0.components.write().unwrap().entry(schedule_id.clone()) {
                Entry::Occupied(exist) => exist.get().clone(),
                Entry::Vacant(empty) => {
                    let newly = Components::from(ManualComponents::new(empty.key()));
                    empty.insert(newly.clone());
                    newly
                }
            }
        };

        if let Some(schdule_component) = component.manual_components() {
            schdule_component.update_schedule(things);
            Ok(())
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, format!("[{schedule_id}] is not maual schedule.")))
        }
    }

    pub(in self) fn update_timeperiod_schedule(&self, 
                                      schedule_id: String, 
                                      cycle_state: CycleState, 
                                      things: Vec<(ThingId, ThingData)>) -> NearResult<()> {
        let component = {
            match self.0.components.write().unwrap().entry(schedule_id.clone()) {
                Entry::Occupied(exist) => exist.get().clone(),
                Entry::Vacant(empty) => {
                    let newly = Components::from(TimePeriodComponents::new(empty.key()));
                    empty.insert(newly.clone());
                    newly
                }
            }
        };

        if let Some(schedule_component) = component.timeperiod_componentes() {
            schedule_component.update_schedule((cycle_state, things));
            Ok(())
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_STATE, format!("[{schedule_id}] is time-period schedule.")))
        }
    }

    pub async fn remove_schedule(&self, schedule_id: &str) -> NearResult<()> {
        let schedule = {
            self.0.components.write().unwrap().remove(schedule_id)
        };

        if let Some(schedule) = schedule {
            match schedule {
                Components::Manual(schedule) => schedule.release().await,
                Components::TimePeriod(schedule) => schedule.release().await,
                Components::Condition(schedule) => schedule.release().await,
            }
            Ok(())
        } else {
            Err(NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found [{schedule_id}] schedule.")))
        }
    }

    pub async fn execute(&self, schedule_id: &str, event: impl OnSchedultEventTrait) -> NearResult<()> {
        let component = {
            self.0.components
                .read().unwrap()
                .get(schedule_id)
                .cloned()
                .ok_or_else(|| {
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found [{schedule_id}] schedule components."))
                })?
        };

        match component {
            Components::Manual(m) => {
                info!("[{} maual execute.]", m.schedule_id());
                m.execute(event).await
            },
            _ => { Err(NearError::new(ErrorCode::NEAR_ERROR_IGNORE, "Ingore execute.")) }
        }
    }

}

impl Manager {
    pub fn on_time_escape(&self) {
        let components: Vec<ScheduleTraitRef<TimePeriodComponents>> = {
            self.0.components.read().unwrap()
                .values()
                .into_iter()
                .filter(| c | {
                    c.timeperiod_componentes().is_some()
                })
                .map(| component | {
                    component.timeperiod_componentes().unwrap()
                })
                .collect()
        };

        let cycle_time_component = self.0.fix_cycle_time_component.clone();

        async_std::task::spawn(async move {
            let mut fut = vec![];
            for component in components.iter() {
                let component_ref = component.clone();

                fut.push( 
                    component.execute(move | thing_dataes: Vec<(ThingId, ThingData)> | {

                        let component_ref = component_ref.clone();

                        async move {
                            trace!("execute [{}] schedule.", component_ref.schedule_id());

                            RoutineTemplate::<Empty>::call(
                                NEAR_THING_SERVICE_SCHEDULE_EXECUTE_PUB.topic().clone(), 
                                (
                                            component_ref.schedule_id().to_owned(),
                                            {
                                                let v: Vec<String> = 
                                                    thing_dataes.into_iter()
                                                        .map(| (thing_id, _) | {
                                                            thing_id.to_string()
                                                        })
                                                        .collect();
                                                v
                                            }
                                        )
                            )
                            .await
                            .map_err(| e | {
                                error!("{e}");
                                e
                            })?
                            .await
                            .map(| _ | ())
                            .map_err(| e | {
                                error!("{e}");
                                e
                            })
                        }
                    })
                );
            }

            fut.push(
                {
                    let cycle_time_component_clone = cycle_time_component.clone();
                    cycle_time_component.execute(move | _: Vec<(ThingId, ThingData)> | {
                        let cycle_time_component = cycle_time_component_clone.clone();

                        async move {
                            trace!("execute [{}] schedule.", cycle_time_component);

                            RoutineTemplate::<Empty>::call(
                                NEAR_THING_SERVICE_QUERY_ALL_THING_PUB.topic().clone(), 
                                Empty
                            )
                            .await
                            .map_err(| e | {
                                error!("{e}");
                                e
                            })?
                            .await
                            .map(| _ | ())
                            .map_err(| e | {
                                error!("{e}");
                                e
                            })

                        }
                    })
                }

            );

            let _ = futures::future::join_all(fut).await;
        });
    }
}

