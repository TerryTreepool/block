
use std::path::PathBuf;

use dataagent_util::*;

fn main() {
    use std::path::Path;

    use protos::brand::Brand_info;

    async_std::task::block_on(async move {
        let db = Helper::new(&PathBuf::new().join("D:\\").join("test_sqlite.db"), None).unwrap();

        pub const CREATE_BRAND: (&str, Option<&str>, Option<&str>, &str) = (
            "create_tb_brand",
            None,
            None,
            r#"create table if not exists tb_brand (
                `brand_id` text primary key not null,
                `brand_name` text not null,
                `status` int default (1)
            );"#
        );
    
        pub const ADD_BRAND: (&str, Option<&str>, Option<&str>, &str) = (
            "add_brand",
            Some("brand_info"),
            None,
            r#"insert into `tb_brand`(`brand_id`, `brand_name`, `status`)
               values(#brand_id#, #brand_name#, #status#);"#
        );

        db.add_sql(CREATE_BRAND.0, CREATE_BRAND.1, CREATE_BRAND.2, CREATE_BRAND.3.to_owned()).unwrap();
        db.add_sql(ADD_BRAND.0, ADD_BRAND.1, ADD_BRAND.2, ADD_BRAND.3.to_owned()).unwrap();

        db.execute(CREATE_BRAND.0).await.unwrap();

        let mut trans = db.begin_transaction().await.unwrap();
        // let transid = TransactionManager::get_instance().begin_transaction(&db).await.unwrap();

        for i in 0..3 {
            // let id = (rand::random::<u32>() % 10u32).to_string();
            let id = 1.to_string();

            

            let brand = 
                Brand_info {
                    brand_id: id,
                    brand_name: format!("name{i}"),
                    status: 1,
                    ..Default::default()
            };

            // TransactionManager::get_instance().execute_with_param(transid, ADD_BRAND.0, &brand).await.unwrap();
            trans.execute_with_param(ADD_BRAND.0, &brand).await
                .map_err(| e | {
                    println!("{e}");
                });
            // fut.push(trans.execute_with_param(ADD_BRAND.0, brand));
        }

        println!("will commit");

        // TransactionManager::get_instance().commit(transid).await.unwrap();
        // let res = futures::future::join_all(fut).await;
        trans.commit().await.unwrap();

        // brand.set_brand_id(22);
        // brand.set_brand_name("xxxyyz".to_owned());
        // brand.set_begin_time("2010-01-01 00:00:00".to_owned());

        // db.execute_with_param(ADD_BRAND.0, &brand).await.unwrap();
    });

}
