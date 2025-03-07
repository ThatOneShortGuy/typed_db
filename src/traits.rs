use rusqlite::Result;
pub trait DbTable {
    const TABLE_NAME: &'static str;
    fn create_table_str() -> String;
    fn column_names() -> Box<[&'static str]>;
    fn create_table(conn: &rusqlite::Connection) -> Result<usize> {
        let sql = Self::create_table_str();
        conn.execute(&sql, ())
    }
}

pub trait DbType {
    fn db_type() -> &'static str;
}
