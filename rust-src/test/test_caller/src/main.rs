use cli_common::api::ApiRequestCommon;

static CS: &'static str = "core-service";

fn main() {
    cli_common::init(None, None).unwrap();
    cli_common::open("BM".to_owned(), CS.to_owned()).unwrap();

    let r = 
        cli_common::query_all_product(
            ApiRequestCommon {
                target: Some("3hAMxnrRmvP5eTxKgz3UwMHVbFbG2B8WYCrkgAHhkeE3".to_owned()),
            }.into(), 
        )
        .unwrap();
    println!("{:?}", r);
    // // add
    // let r = crate::add_device("4D9E67D78509139B1C7329E5F53587C9".to_owned(),
    //                     "00-50-56-C0-00-08".to_owned(),
    //                    "test_device_a".to_owned(),
    //                    1);

    println!("++++++++++++++++++++++++++++++++++++++++++++++");
}
