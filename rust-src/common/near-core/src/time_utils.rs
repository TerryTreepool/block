
use chrono::{NaiveDateTime, DateTime, Local};

pub fn native_now() -> NaiveDateTime {
    let r = std::time::SystemTime::now();
    let r: DateTime<Local>  = r.into();
    r.naive_local()
}

mod test {

    #[test]
    pub fn test_native_time() {
        let nt = super::native_now();
        // let nt_rfc3339 = nt.to_rfc3339();
        // let nt_rfc2822 = nt.to_rfc2822();
        // let nt_string = nt.to_string();
        // let nt_dt = nt.naive_local();
        // nt_dt.
        let nt_format = nt.format("%Y-%m-%d %H:%M:%S").to_string();

        // println!("{nt_rfc3339}");
        // println!("{nt_rfc2822}");
        // println!("{nt_string}");
        println!("{nt_format}");
        println!("{}", nt.to_string());
    }
}
