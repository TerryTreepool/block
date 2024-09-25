
use std::path::Path;

use near_base::{NearResult, ErrorCode, NearError};
use protobuf::MessageDyn;
use sqlx::{Row, Column};

use crate::Transaction;

use super::{p::Manager, sqlx_sqlite::{Pool, Config, SqlArguments, SqlRowObject, SqlTransaction, SqlStateWrapper}};

pub struct Helper {
    manager: Manager,
    pool: Pool
}

impl Helper {
    pub fn new(db: &Path, config: Option<Config>) -> NearResult<Self> {
        let db = db.to_str()
                         .ok_or_else(|| NearError::new(ErrorCode::NEAR_ERROR_INVALIDPARAM, "db invalid path."))?;

        Ok(Self {
            manager: Manager::get_instance().clone(),
            pool: Pool::open(format!("sqlite://{}", db).as_str(), config.unwrap_or_default())?,
        })
    }

    pub fn add_sql(&self, 
                   sql_id: impl std::string::ToString, 
                   input_param: Option<&str>, 
                   output_param: Option<&str>,
                   sql: String) -> NearResult<()> {
        self.manager.add_sql(sql_id.to_string(), input_param, output_param, sql)
    }

    pub async fn begin_transaction(&self) -> NearResult<Transaction> {
        Transaction::new(SqlTransaction::begin(&self.pool).await?)
    }
}

impl Helper {

    pub(crate) fn bind_variable<'a>(sql: SqlArguments<'a>, 
                                   bind_field_name: &str, 
                                   bind_params: &'a impl protobuf::MessageFull) -> NearResult<SqlArguments<'a>> {
        let field_descriptor = 
            bind_params.descriptor_dyn()
                .field_by_name(bind_field_name)
                .ok_or_else(|| {
                    NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found {bind_field_name} in Message."))
                })?;

        let field_value = 
            match field_descriptor.get_reflect(bind_params) {
                protobuf::reflect::ReflectFieldRef::Optional(v) => {
                    Ok(v.value())
                }
                protobuf::reflect::ReflectFieldRef::Map(_v) => {
                    Err(NearError::new(ErrorCode::NEAR_ERROR_PROTOC_NOT_SUPPORT_MAP, "Not support map"))
                }
                protobuf::reflect::ReflectFieldRef::Repeated(_v) => {
                    Err(NearError::new(ErrorCode::NEAR_ERROR_PROTOC_NOT_SUPPORT_REPEATED, "Not support repeated"))
                }
        }?;

        let sql = 
            if let Some(field_value) = field_value {
                match field_value {
                    protobuf::reflect::ReflectValueRef::Bool(v)    => sql.bind(v),
                    protobuf::reflect::ReflectValueRef::F64(v)      => sql.bind(v),
                    protobuf::reflect::ReflectValueRef::Bytes(v)  => sql.bind(v),
                    protobuf::reflect::ReflectValueRef::F32(v)      => sql.bind(v),
                    protobuf::reflect::ReflectValueRef::I32(v)      => sql.bind(v),
                    protobuf::reflect::ReflectValueRef::I64(v)      => sql.bind(v),
                    protobuf::reflect::ReflectValueRef::String(v)  => sql.bind(v),
                    protobuf::reflect::ReflectValueRef::U32(v)      => sql.bind(v as i64),
                    protobuf::reflect::ReflectValueRef::U64(v)      => sql.bind(v as i64),
                    protobuf::reflect::ReflectValueRef::Message(_) | 
                    protobuf::reflect::ReflectValueRef::Enum(_, _) => { unimplemented!() },
                }
            } else {
                match field_descriptor.proto().type_() {
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_DOUBLE |
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_FLOAT => {
                        sql.bind::<Option<f32>>(None)
                    }
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_INT64 | 
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_UINT32 | 
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_UINT64 => {
                        sql.bind::<Option<i64>>(None)
                    }
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_INT32 => {
                        sql.bind::<Option<i32>>(None)
                    }
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_BOOL => {
                        sql.bind::<Option<bool>>(None)
                    }
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_STRING => {
                        sql.bind::<Option<&str>>(None)
                    }
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_BYTES => {
                        sql.bind::<Option<&[u8]>>(None)
                    }
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_FIXED64 | 
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_FIXED32 |
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_GROUP | 
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_MESSAGE |
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_ENUM |
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_SFIXED32 |
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_SFIXED64 | 
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_SINT32 |
                    protobuf::descriptor::field_descriptor_proto::Type::TYPE_SINT64 => {
                        unreachable!()
                    }
                }
            };

        Ok(sql)
    }

    pub(crate) fn get_variable<T>(row: &SqlRowObject) -> NearResult<Box<T>>
    where T: protobuf::MessageFull {
        let message_descriptor = T::descriptor();
        let mut message_instance = message_descriptor.new_instance();

        for i in row.columns() {
            let field_name = i.name();

            let field = 
                message_descriptor.field_by_name(field_name)
                    .ok_or_else(|| {
                        NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("Not found {field_name} in Message."))
                    })?;

            match field.proto().type_() {
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_DOUBLE => {
                    let v: f64 = row.get(field_name);
                    field.set_singular_field(message_instance.as_mut(), v.into());
                }
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_FLOAT => {
                    let v: f64 = row.get(field_name);
                    field.set_singular_field(message_instance.as_mut(), v.into());
                }
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_INT64 => {
                    let v: i64 = row.get(field_name);
                    field.set_singular_field(message_instance.as_mut(), v.into());
                }
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_UINT32 => {
                    let v: i64 = row.get(field_name);
                    field.set_singular_field(message_instance.as_mut(), (v as u32).into());
                }
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_UINT64 => {
                    let v: i64 = row.get(field_name);
                    field.set_singular_field(message_instance.as_mut(), (v as u64).into());
                }
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_INT32 => {
                    let v: i32 = row.get(field_name);
                    field.set_singular_field(message_instance.as_mut(), v.into());
                }
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_BOOL => {
                    let v: bool = row.get(field_name);
                    field.set_singular_field(message_instance.as_mut(), v.into());

                }
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_STRING => {
                    let v: String = row.get(field_name);
                    field.set_singular_field(message_instance.as_mut(), v.into());

                }
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_BYTES => {
                    let v: Vec<u8> = row.get(field_name);
                    field.set_singular_field(message_instance.as_mut(), v.into());
                }
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_FIXED64 | 
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_FIXED32 |
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_GROUP | 
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_MESSAGE |
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_ENUM |
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_SFIXED32 |
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_SFIXED64 | 
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_SINT32 |
                protobuf::descriptor::field_descriptor_proto::Type::TYPE_SINT64 => {
                    unreachable!()
                }
            }
        
        }

        let dynmaic_message_instance = 
            message_instance.downcast_box::<T>().unwrap();

        Ok(dynmaic_message_instance)
    }

    pub async fn query_all<T>(&self, 
                              sql_id: &str) -> NearResult<Vec<T>>
    where T: protobuf::MessageFull {
        let mut conn = self.pool.get_conn().await?;

        SqlStateWrapper::query_all(&mut conn, sql_id)
            .await
    }

    pub async fn query_all_with_param<T>(&self, 
                                         sql_id: &str, 
                                         params: impl protobuf::MessageFull) -> NearResult<Vec<T>>
    where T: protobuf::MessageFull {
        let mut conn = self.pool.get_conn().await?;

        SqlStateWrapper::query_all_with_param(&mut conn, sql_id, params)
            .await
    }

    pub async fn execute(&self, sql_id: &str) -> NearResult<()> {
        let mut conn = self.pool.get_conn().await?;

        SqlStateWrapper::execute(&mut conn, sql_id)
            .await
    }

    pub async fn execute_with_param(&self, sql_id: &str, params: &impl protobuf::MessageFull) -> NearResult<()> {
        let mut conn = self.pool.get_conn().await?;

        SqlStateWrapper::execute_with_param(&mut conn, sql_id, params)
            .await
    }

}

mod test {
   
    #[test]
    fn test_insert_message() {
        use std::path::Path;

        use protos::brand::Brand_info;
    
        use crate::Helper;
    
        async_std::task::block_on(async move {
            let db = Helper::new(&Path::new("/Users/tianzhuyan").join("hci-manager.db"), None).unwrap();

            pub const ADD_BRAND: (&str, &str) = (
                "add_brand",
                r#"insert into `tb_brand`(`brand_id`, `brand_name`)
                   values(#brand_id#, #brand_name#);"#
            );
        
            db.add_sql(ADD_BRAND.0, None, None, ADD_BRAND.1.to_owned()).unwrap();
    
            let mut brand = Brand_info::new();
            brand.set_brand_id(22.to_string());
            brand.set_brand_name("xxxyyz".to_owned());
            // brand.set_begin_time("2010-01-01 00:00:00".to_owned());
    
            db.execute_with_param(ADD_BRAND.0, &brand).await.unwrap();
        });
    }

    #[test]
    fn test_query_message() {
        use std::path::Path;

        use protobuf::Message;
        use protos::brand::Brand_info;
    
        use crate::Helper;
    
        async_std::task::block_on(async move {
            let db = Helper::new(&Path::new("/Users/tianzhuyan").join("hci-manager.db"), None).unwrap();

            pub const QUERY_BRAND: (&str, &str) = (
                "query_brand",
                r#"select brand_id, brand_name, status, begin_time from tb_brand limit 100;"#
            );
        
            db.add_sql(QUERY_BRAND.0, None, Some("brand_info"), QUERY_BRAND.1.to_owned()).unwrap();

            let all = db.query_all::<Brand_info>(QUERY_BRAND.0).await.unwrap();

            for it in all {
                println!("{:?}", it.write_to_bytes().unwrap());
            }
        });
    }

}
