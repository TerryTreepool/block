
pub const SERVICE_NAME: &str = "hci-manager";

pub const CREATE_BRAND: (&str, Option<&str>, Option<&str>, &str) = (
    "create_tb_brand",
    None,
    None,
    r#"create table if not exists tb_brand (
        `brand_id` text primary key not null,
        `brand_name` text not null,
        `begin_time` timestamp not null default (datetime('now', 'localtime')),
        `update_time` timestamp not null default (datetime('now', 'localtime')),
        `status` int default (1)
    );"#
);

pub const CREATE_PRODUCT: (&str, Option<&str>, Option<&str>, &str) = (
    "create_tb_product",
    None,
    None,
    r#"create table if not exists tb_product (
        `parent_product_id` text,
        `product_id` text primary key not null,
        `product_name` text not null,
        `begin_time` timestamp not null default (datetime('now', 'localtime')),
        `update_time` timestamp not null default (datetime('now', 'localtime')),
        `status` int default (1)
    );"#
);

pub const CREATE_DEVICE: (&str, Option<&str>, Option<&str>, &str) = (
    "create_tb_device",
    None,
    None,
    r#"create table if not exists tb_device (
        `brand_id` text,
        `product_id` text,
        `device_id` text primary key not null,
        `device_name` text not null,
        `mac_address` text not null,
        `begin_time` timestamp not null default (datetime('now', 'localtime')),
        `update_time` timestamp not null default (datetime('now', 'localtime')),
        `status` int default (1)
    );"#
);

pub const CREATE_THING_GROUP: (&str, Option<&str>, Option<&str>, &str) = (
    "create_tb_group",
    None,
    None,
    r#"create table if not exists tb_group (
        `group_id` text primary key not null,
        `group_name` text not null,
        `begin_time` timestamp not null default (datetime('now', 'localtime')),
        `update_time` timestamp not null default (datetime('now', 'localtime')),
        `status` int default (1)
    );"#
);

pub const CREATE_GROUP_RELATION: (&str, Option<&str>, Option<&str>, &str) = (
    "create_tb_group_relation",
    None,
    None,
    r#"create table if not exists tb_group_relation (
        `group_id` text not null,
        `thing_id` text not null,
		`thing_data_property_text` text,
        `begin_time` timestamp not null default (datetime('now', 'localtime')),
        `update_time` timestamp not null default (datetime('now', 'localtime')),
        `status` int default (1),
        PRIMARY KEY (group_id, thing_id)
    );"#
);

// brand
pub const ADD_BRAND: (&str, Option<&str>, Option<&str>, &str) = (
    "add_brand",
    Some("brand_info"),
    None,
    r#"insert into `tb_brand`(`brand_id`, `brand_name`, `begin_time`, `update_time`)
       values(#brand_id#, #brand_name#, #begin_time#, #update_time#);"#
);

pub const UPDATE_BRAND: (&str, Option<&str>, Option<&str>, &str) = (
    "update_brand",
    Some("brand_info"),
    None,
    r#"update `tb_brand` 
       set `status` = #status#, 
           `update_time` = #update_time# 
       where `brand_id` = #brand_id#;"#
);

pub const GET_ALL_BRAND: (&str, Option<&str>, Option<&str>, &str) = (
    "get_all_brand",
    None,
    Some("brand_info"),
    r#"select `brand_id`, `brand_name`, `begin_time`, `status` from tb_brand;"#
);

pub const GET_BRAND: (&str, Option<&str>, Option<&str>, &str) = (
    "get_all_brand",
    Some("brand_info"),
    Some("brand_info"),
    r#"select `brand_id`, `brand_name`, `begin_time`, `update_time`, `status` 
       from tb_brand
       where brand_id=#brand_id#;"#
);

/// product
pub const GET_ALL_PRODUCT: (&str, Option<&str>, Option<&str>, &str) = (
    "get_all_product",
    None,
    Some("product_info"),
    r#"select `parent_product_id`,
              `product_id`,
              `product_name`,
              `begin_time`,
              `update_time`,
              `status`
       from tb_product;"#,
);

/// product
pub const GET_PRODUCT: (&str, Option<&str>, Option<&str>, &str) = (
    "get_all_product",
    Some("product_info"),
    Some("product_info"),
    r#"select `parent_product_id`,
              `product_id`,
              `product_name`,
              `begin_time`,
              `update_time`,
              `status`
       from tb_product;
       where product_id=#product_id#"#,
);

pub const ADD_PRODUCT: (&str, Option<&str>, Option<&str>, &str) = (
    "add_product",
    Some("product_info"),
    None,
    r#"insert into `tb_product`(`parent_product_id`, `product_id`, `product_name`, `begin_time`, `update_time`, `status`)
       values(#parent_product_id#, #product_id#, #product_name#, #begin_time#, #update_time#, #status#);"#
);

pub const UPDATE_PRODUCT: (&str, Option<&str>, Option<&str>, &str) = (
    "update_product",
    Some("product_info"),
    None,
    r#"update `tb_product` 
    set `status` = #status#, 
        `update_time` = #update_time# 
    where `product_id` = #product_id#;"#
);

/// device 
pub const GET_ALL_DEVICE: (&str, Option<&str>, Option<&str>, &str) = (
    "get_all_device",
    Some("device_info"),
    Some("device_info"),
    r#"select `brand_id`, 
              `product_id`,
              `device_id`,
              `device_name`,
              `mac_address`,
              `begin_time`,
              `update_time`,
              `status`
       from tb_device
       where `product_id` = ifnull(#product_id#, `product_id`) and
             `device_id` = ifnull(#device_id#, `device_id`) and
             `brand_id` = ifnull(#brand_id#, `brand_id`);"#,
);

pub const ADD_DEVICE: (&str, Option<&str>, Option<&str>, &str) = (
    "add_device",
    Some("device_info"),
    None,
    r#"insert into `tb_device`(`brand_id`, 
                               `product_id`, 
                               `device_id`, `device_name`, `mac_address`, 
                               `begin_time`, `update_time`, `status`)
       values(#brand_id#, #product_id#, #device_id#, #device_name#, #mac_address#, #begin_time#, #update_time#, #status#);"#
);

pub const UPDATE_DEVICE: (&str, Option<&str>, Option<&str>, &str) = (
    "update_device",
    Some("device_info"),
    None,
    r#"update `tb_device` 
    set `device_name` = IFNULL(#device_name#, `device_name`),
        `status` = IFNULL(#status#, `status`), 
        `update_time` = #update_time# 
    where `device_id` = #device_id#;"#
);

// group
pub const ADD_GROUP: (&str, Option<&str>, Option<&str>, &str) = (
    "add_group",
    Some("thing_group_info"),
    None,
    r#"insert into `tb_group`(`group_id`, `group_name`, `begin_time`, `update_time`, `status`)
        values(#group_id#, #group_name#, #begin_time#, #update_time#, #status#);"#
);

pub const UPDATE_GROUP: (&str, Option<&str>, Option<&str>, &str) = (
    "update_group",
    Some("thing_group_info"),
    None,
    r#"update `tb_group` 
          set `status` = #status#, 
              `update_time` = #update_time# 
       where `group_id` = #group_id#;"#
);

pub const GET_ALL_GROUP: (&str, Option<&str>, Option<&str>, &str) = (
    "get_all_group",
    None,
    Some("thing_group_info"),
    r#"select `group_id`, `group_name`, `begin_time`, `update_time`, `status` from tb_group;"#
);

pub const GET_GROUP: (&str, Option<&str>, Option<&str>, &str) = (
    "get_group",
    Some("thing_group_info"),
    Some("thing_group_info"),
    r#"select `group_id`, `group_name`, `begin_time`, `update_time`, `status` 
       from tb_group
       where `group_id` = #group_id#;"#
);

// group an thing relation
pub const ADD_THING_GROUP_RELATION: (&str, Option<&str>, Option<&str>, &str) = (
    "add_group_relation",
    Some("thing_group_relation_info"),
    None,
    r#"
    insert into `tb_group_relation`(`group_id`, `thing_id`, `thing_data_property_text`, `begin_time`, `update_time`, `status`)
    values(#group_id#, #thing_id#, #thing_data_property_text#, #begin_time#, #update_time#, #status#);"#
);

pub const UPDATE_THING_GROUP_RELATION: (&str, Option<&str>, Option<&str>, &str) = (
    "update_group_relation",
    Some("thing_group_relation_info"),
    None,
    r#"
    update `tb_group_relation`
    set `thing_data_property_text` = ifnull(#thing_data_property_text#, `thing_data_property_text`),
        `update_time` = #update_time#
    where `group_id`=#group_id# and `thing_id`=#thing_id#;"#
);

pub const DELETE_THING_GROUP_RELATION: (&str, Option<&str>, Option<&str>, &str) = (
    "delete_group_relation",
    Some("thing_group_relation_info"),
    None,
    r#"delete from `tb_group_relation` where `group_id`=#group_id# and `thing_id`=#thing_id#;"#
);

pub const QUERY_ALL_THING_GROUP_RELATION: (&str, Option<&str>, Option<&str>, &str) = (
    "query_all_group_relation",
    Some("thing_group_relation_info"),
    Some("thing_group_relation_info"),
    r#"
    select `group_id`, `thing_id`, `thing_data_property_text`, `status` 
    from tb_group_relation
    where group_id=#group_id#;"#
);

pub const QUERY_THING_GROUP_RELATION: (&str, Option<&str>, Option<&str>, &str) = (
    "query_group_relation",
    Some("thing_group_relation_info"),
    Some("thing_group_relation_info"),
    r#"
    select `group_id`, `thing_id`, `thing_data_property_text`, `status` 
    from tb_group_relation
    where group_id=#group_id# and thing_id=#thing_id#;"#
);
