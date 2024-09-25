use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDate, NaiveTime, Timelike, Weekday};
use crossbeam::epoch::{self, Atomic, Owned, Shared};
use instant::now;

fn week_bounds(week: u32) -> (NaiveDate, NaiveDate) {
    let current_year = chrono::offset::Local::now().year();
    let mon = NaiveDate::from_isoywd(current_year, week, Weekday::Mon);
    let sun = NaiveDate::from_isoywd(current_year, week, Weekday::Sun);
    (mon, sun)
}

fn main() {

    {
        let n = Atomic::<u64>::default();

        let ep = &epoch::pin();
        let r = 
        n.compare_exchange(
            Shared::null(),
              Owned::new(100),
              std::sync::atomic::Ordering::SeqCst, 
              std::sync::atomic::Ordering::SeqCst,
              ep);
        println!("r={:?}", r);
        let v = n.load(std::sync::atomic::Ordering::SeqCst, ep);
        println!("{:?}", unsafe {n.load(std::sync::atomic::Ordering::SeqCst, ep).as_ref()});
    }

    {
        let mut plus = 0;

        let be = near_base::now();
        for _ in 0..100000 {
            async_std::task::block_on(async move {
                plus += 1;
            });
        }
        let ed = near_base::now();
        println!("{}", std::time::Duration::from_micros(ed-be).as_millis());

        std::process::exit(0);
    }
    {
        let mut mapes = BTreeMap::new();
        mapes.insert(1, "a1".to_owned());
        mapes.insert(2, "a2".to_owned());
        mapes.insert(3, "a3".to_owned());
        mapes.insert(4, "a4".to_owned());
        mapes.insert(5, "a5".to_owned());
        mapes.insert(6, "a6".to_owned());
        mapes.insert(7, "a7".to_owned());

        println!("{:?}", mapes);

        // let mut set: BTreeSet<i32> = (0..=10).collect();
        mapes.retain(| &k, v | { k%2==0 });
// assert_eq!(set.into_iter().collect::<Vec<_>>(), [0, 1, 2, 3, 4, 9, 10]);
// assert_eq!(drained, [5, 6, 7, 8]);
        // let t: HashMap<i32, String> = 
        //     mapes.drain().into_iter()
        //         .filter(| (k, _) | { k % 2 == 0 })
        //         .take(10)
        //         .collect();
        // let mut_it = 
        //     mapes.iter_mut()
        //         .filter(| (&k, v) | k%2==0 )
        //         ;

        // let take = mut_it.take(2);

        // println!("{:?}", t);
        println!("{:?}", mapes);

    }
    {
        let (a, b) = week_bounds(10);
        println!("{}", a);
        println!("{}", b);

        let now_time = chrono::Local::now();
        enum Time {
            AM(NaiveTime),
            PM(NaiveTime),
        }

        impl std::fmt::Display for Time {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::AM(v) => write!(f, "01{:02X}{:02X}{:02X}", v.hour(), v.minute(), v.second()),
                    Self::PM(v) => write!(f, "02{:02X}{:02X}{:02X}", v.hour(), v.minute(), v.second()),
                }
            }
        }
        let (v, hour) = now_time.hour12();
        let minutes = now_time.minute();
        let seconds = now_time.second();
        let v = 
            match v {
                true => Time::PM(NaiveTime::from_hms_opt(hour, now_time.minute(), now_time.second()).unwrap()),
                false => Time::AM(NaiveTime::from_hms_opt(hour, now_time.minute(), now_time.second()).unwrap()),
            };

        println!("+++++++++{v}");

        println!("========={}", now_time);

        let t1 = chrono::Local::now().date_naive().weekday();

        println!("{}", t1);

        // let timezone_east = FixedOffset::east_opt(8 * 60 * 60).unwrap();
        // let naivedatetime_east = NaiveDate::from_ymd_opt(2000, 1, 12).unwrap().and_hms_opt(10, 0, 0).unwrap();
        let time = chrono::NaiveDate::default().and_hms_opt(13, 40, 0).unwrap();
        // let t2 = chrono::DateTime::<FixedOffset>::from_local(naivedatetime_east, timezone_east);
        let t2 = chrono::NaiveTime::from_hms_opt(13, 40, 0).unwrap();
        let now_time = chrono::Local::now()
            .time()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        let t2_n = now_time.signed_duration_since(t2);
        println!("{}, {now_time}, {t2_n}", t2);

        let now = chrono::Local::now();
    }

    {
        // let in1 = instant::Instant::now().
    }

    // NaiveDate::from_isoywd_opt(year, week, weekday)
    // println!("{}", t2);
}
