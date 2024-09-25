
use std::{sync::{RwLock, Arc}, collections::{BTreeMap, btree_map::Entry}, };

use log::{warn, info};
use near_base::{NearError, ErrorCode, NearResult};

struct SqlStructImpl {
    sql: String,
    sql_params: Vec<String>,

    input_param: Option<String>,
    output_param: Option<String>,
}

#[derive(Clone)]
pub struct SqlStruct(Arc<SqlStructImpl>);

impl SqlStruct {
    #[inline]
    pub fn sql(&self) -> &str {
        self.0.sql.as_str()
    }

    #[inline]
    pub fn sql_params(&self) -> &Vec<String> {
        &self.0.sql_params
    }

    #[inline]
    #[allow(unused)]
    pub fn input_param(&self) -> Option<&str> {
        self.0.input_param.as_ref().map(| p | p.as_str())
    }

    #[inline]
    pub fn set_input_param(&mut self, input_param: Option<String>) {
        let mut_self = unsafe { &mut *(Arc::as_ptr(&self.0) as *mut SqlStructImpl) };
        mut_self.input_param = input_param;
    }

    #[inline]
    #[allow(unused)]
    pub fn output_param(&self) -> Option<&str> {
        self.0.output_param.as_ref().map(| p | p.as_str())
    }

    #[inline]
    pub fn set_output_params(&mut self, output_param: Option<String>) {
        let mut_self = unsafe { &mut *(Arc::as_ptr(&self.0) as *mut SqlStructImpl) };
        mut_self.output_param = output_param;
    }
}

impl TryFrom<String> for SqlStruct {
    type Error = NearError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let count = value.len();
        let mut sql = String::new();
        let mut sql_params = vec![];

        unsafe {
            let ptr = value.as_ptr();
            let mut index = 0usize;

            let index_of = | index: usize | {
                if index < count { ptr.offset(index as isize).as_ref() } else { None }
            };

            while let Some(ch) = index_of(index) {
                if *ch == b'#' {
                    sql.push('?');
                    index = index + 1;

                    let key = {
                        let orig_index = index;
                        while let Some(ch) = index_of(index) {
                            if *ch == b'#' {
                                break;
                            }
                            index = index + 1;
                        }

                        let len = (index - orig_index) as usize;
                        let mut buf = vec![];
                        buf.resize(len, 0u8);
                        std::ptr::copy(ptr.offset(orig_index as isize), buf.as_mut_ptr(), len);

                        String::from_utf8(buf)
                            .map_err(| err | {
                                NearError::new(ErrorCode::NEAR_ERROR_SYSTERM, err.to_string())
                            })
                    }?;

                    index = index + 1;
                    sql_params.push(key);
                } else {
                    sql.push(*ch as char);
                    index = index + 1;
                }
            }
        }

        Ok(Self(Arc::new(SqlStructImpl{
            sql, 
            sql_params, 
            input_param: None,
            output_param: None,
        })))
    }
}

struct ManagerImpl {
    items: RwLock<BTreeMap<String, SqlStruct>>,
}

#[derive(Clone)]
pub struct Manager(Arc<ManagerImpl>);

impl Manager {
    pub fn get_instance() -> &'static Manager {
        static INSTANCE: once_cell::sync::OnceCell<Manager> = once_cell::sync::OnceCell::new();

        INSTANCE.get_or_init(|| {
            let r = Self(Arc::new(ManagerImpl{
                items: RwLock::new(BTreeMap::new()),
            }));

            r
        })
    }

    pub fn add_sql(&self, sql_id: String, input_param: Option<&str>, output_param: Option<&str> ,sql: String) -> NearResult<()> {
        let items = &mut *self.0.items.write().unwrap();
        match items.entry(sql_id) {
            Entry::Occupied(_exist) => {
                // ignore
                warn!("[{}] existed.", _exist.key());
            }
            Entry::Vacant(not_found) => {
                info!("[{}] added.", not_found.key());
                let mut sql_struct = SqlStruct::try_from(sql)?;
                sql_struct.set_input_param(input_param.map(| p | p.to_owned()));
                sql_struct.set_output_params(output_param.map(| p | p.to_owned()));
                not_found.insert(sql_struct);
            }
        }

        Ok(())
    }

    pub fn get_sql(&self, sql_id: &str) -> NearResult<SqlStruct> {
        self.0.items.read().unwrap()
            .get(sql_id)
            .map(| item | item.clone() )
            .ok_or_else(|| {
                NearError::new(ErrorCode::NEAR_ERROR_NOTFOUND, format!("[{}] not found.", sql_id))
            })
    }
}

