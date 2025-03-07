use chrono::prelude::*;

use crate::DbType;

macro_rules! impl_db_type {
    ($($ty:ty) *, $db_type:expr) => {
        $(
        impl DbType for $ty {
            fn db_type() -> &'static str {
                $db_type
            }
        })*
    };
}

impl_db_type!(i8 u8, "TINYINT");
impl_db_type!(i16 u16, "SMALLINT");
impl_db_type!(i32 u32, "INTEGER");
impl_db_type!(i64, "BIGINT");
impl_db_type!(u64, "UNSIGNED BIG INT");
impl_db_type!(&str String std::path::PathBuf std::path::Path, "TEXT");
impl_db_type!(f32, "FLOAT");
impl_db_type!(f64, "DOUBLE");
impl_db_type!(bool, "BOOLEAN");
impl_db_type!(NaiveDate, "DATE");
impl_db_type!(NaiveDateTime DateTime<Utc>, "DATETIME");
impl_db_type!(Vec<u8>, "BLOB");

impl<T: DbType> DbType for Option<T> {
    fn db_type() -> &'static str {
        T::db_type()
    }
}

#[cfg(test)]
mod tests {

    use crate::DbTable;

    use super::*;

    #[test]
    fn create_table_test() {
        #[allow(dead_code)]
        #[derive(Debug, DbTable)]
        struct TestTable {
            #[primary_key]
            pub id: u32,
            pub name: String,
            #[unique]
            pub email: String,
            pub password_hash: String,
            #[default(CURRENT_TIMESTAMP)]
            pub created_date: DateTime<Utc>,
        }
        let conn = rusqlite::Connection::open(":memory:").unwrap();
        println!("{}", TestTable::create_table_str());
        TestTable::create_table(&conn).unwrap();

        TestTable::new()
            .with_email("test@test.com".to_string())
            .with_password_hash("kalsdjfalskdfja".to_string())
            .with_name("namessss".to_string())
            .build(&conn)
            .unwrap();

        let d = TestTable::select(&conn, "WHERE email GLOB \"*test*\"", []).unwrap();

        panic!("{:?}", d)
        // panic!("{}", TestTable::create_table_str())
    }
}
