use rusqlite::Result;

pub trait DbTable: Sized {
    const TABLE_NAME: &'static str;
    fn create_table_str() -> String;
    fn column_names() -> Box<[&'static str]>;
    fn create_table(conn: &rusqlite::Connection) -> Result<usize> {
        let sql = Self::create_table_str();
        conn.execute(&sql, ())
    }
    fn select(
        conn: &rusqlite::Connection,
        where_clause: &str,
        params: impl rusqlite::Params,
    ) -> rusqlite::Result<Box<[Self]>>;
}

pub trait DbType: Default {
    fn db_type() -> &'static str;
}

pub trait CommonTableExpression: Sized {
    fn cte_str() -> &'static str;
    fn select(
        conn: &rusqlite::Connection,
        params: impl rusqlite::Params,
    ) -> rusqlite::Result<Box<[Self]>>;
}
