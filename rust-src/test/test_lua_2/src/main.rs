use std::{path::PathBuf, io::Read};

use near_base::now;


mod lua;

#[async_std::main]
async fn main() {

    let params: Vec<String> = std::env::args().skip(1).collect();

    if params.len() == 0 {
        panic!("missing runner script.");
    }

    let script = params.get(0).unwrap();
    let m = lua::manager::Manager::open(PathBuf::new().join("D:\\p\\near\\rust-src\\deploy\\lua")).await.unwrap();

    // let v = vec![]
    let data = {
        let mut fs = 
            std::fs::OpenOptions::new()
                .read(true)
                .open(PathBuf::from(script).as_path())
                .expect(format!("{}", script).as_str());

        let mut data = vec![0u8; fs.metadata().unwrap().len() as usize];
        fs.read_exact(data.as_mut_slice());
        let mut array = vec![];

        for iter in data.split(| c | *c == b'\n') {
            if let Ok(v) = hex::decode(iter.to_vec()) {
                array.push(v);
            }
        }

        array
    };

    let count = data.len();
    let begin = now();

    let mut array = vec![];

    for i in 0..200000 {
        let one = data.get(i%count).unwrap().clone();
        let m_clone = m.clone();
        let h = async_std::task::spawn(async move {
            match m_clone.analyze_data(one).await {
                Ok(d) => { 
                    for (k, v) in d.into_map() {
                        println!("{k}={v}");
                    }
                }
                Err(e) => { println!{"-"} }
            }    
        });

        array.push(h);

        if array.len() >= 1 {
            let mut join_array = vec![];
            std::mem::swap(&mut array, &mut join_array);
            let _ = futures::future::join_all(join_array).await;
        }
    }

    if array.len() >= 0 {
        let mut join_array = vec![];
        std::mem::swap(&mut array, &mut join_array);
        let _ = futures::future::join_all(join_array).await;
    }

    let end = now();
    print!("\n");

    let p = std::time::Duration::from_millis(end-begin);
    println!("time expire: {}", p.as_secs());
}

#[test]
fn test_vec() {
    let v = vec![0, 1, 2, 3, 4];

    let v1: Vec<i32> = v.iter()
                        .filter(|&i| i%2==0 )
                        .cloned()
                        .collect();

    println!("{:?}", v1);
    println!("{:?}", v);
}

#[test]
fn test_hex() {
    let v = "B154A0E2FF01F511111111780EC101026432F5F5F5F5F5F5F5F5F5F5F5";

    let r = hex::decode(v).unwrap();

    println!("{:?}", r);
}

