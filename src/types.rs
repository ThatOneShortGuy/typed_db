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
impl_db_type!(i32 u32 isize usize, "INTEGER");
impl_db_type!(i64, "BIGINT");
impl_db_type!(u64, "UNSIGNED BIG INT");
impl_db_type!(&str String std::path::PathBuf, "TEXT");
impl_db_type!(f32, "FLOAT");
impl_db_type!(f64, "DOUBLE");
impl_db_type!(bool, "BOOLEAN");
impl_db_type!(NaiveDate, "DATE");
impl_db_type!(NaiveDateTime DateTime<Utc>, "DATETIME");
impl_db_type!(Vec<u8> &[u8], "BLOB");

impl<T: DbType> DbType for Option<T> {
    fn db_type() -> &'static str {
        T::db_type()
    }
}
