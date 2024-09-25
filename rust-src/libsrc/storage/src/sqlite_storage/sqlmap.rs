
use super::StorageType;

pub(super) struct SqlmapBuild<'a> {
    pub(super) name: &'a str,
    pub(super) mode: StorageType,
}

impl SqlmapBuild<'_> {
    pub(super) fn build_sql_key(self, ) -> String {
        match &self.mode {
            StorageType::Init => format!("init_{}_key", self.name),
            StorageType::QueryOne => format!("query_{}_key", self.name),
            StorageType::QueryAll => format!("queryall_{}_key", self.name),
            StorageType::Create => format!("create_{}_key", self.name),
            StorageType::Update => format!("update_{}_key", self.name),
            StorageType::Delete => format!("delete_{}_key", self.name),
        }
    }

    pub(super) fn build(self) -> (String, Option<String>, Option<String>, String) {
        let mode = self.mode;
        let name = self.name;

        let key = self.build_sql_key();
        match mode {
            StorageType::Init => {
                let sql = 
                    format!(r#"CREATE TABLE IF NOT EXISTS {} (
                                key TEXT PRIMARY KEY NOT NULL UNIQUE, 
                                value BLOB NOT NULL
                            );"#, name);
                (key, None, None, sql)
            }
            StorageType::QueryOne => {
                let sql = format!("select key, value from {} where key = #key#", name);

                (key, None, Some("data".to_owned()), sql)
            }
            StorageType::QueryAll => {
                let sql = format!(r#"select key, value from {}"#, name);

                (key, None, Some("data".to_owned()), sql)
            }
            StorageType::Create => {
                let sql = format!(r#"insert into {} (key, value) values (#key#, #value#)"#, name);

                (key, Some("data".to_owned()), None, sql)
            }
            StorageType::Update => {
                let sql = format!("update {} set value = #value# where key = #key#", name);

                (key, Some("data".to_owned()), None, sql)
            }
            StorageType::Delete => {
                let sql = format!("delete from {} where key = #key#", name);

                (key, Some("data".to_owned()), None, sql)
            }
        }
    }
}
