use rusqlite::{OptionalExtension, Result};

pub trait DbTable: Sized + for<'a> TryFrom<&'a rusqlite::Row<'a>>
where
    rusqlite::Error: for<'a> From<<Self as TryFrom<&'a rusqlite::Row<'a>>>::Error>,
{
    const TABLE_NAME: &'static str;
    fn create_table_str() -> String;
    fn column_names() -> Box<[&'static str]>;
    fn column_getters() -> String {
        Self::column_names().join(",")
    }
    /// Create the table in the database.
    fn create_table(conn: &rusqlite::Connection) -> Result<usize> {
        let sql = Self::create_table_str();
        conn.execute(&sql, ())
    }

    /// Selects all rows from the table for which the where clause is true.
    fn select(
        conn: &rusqlite::Connection,
        where_clause: &str,
        params: impl rusqlite::Params,
    ) -> rusqlite::Result<Box<[Self]>> {
        let sql = format!(
            "SELECT {} FROM {} {}",
            Self::column_getters(),
            Self::TABLE_NAME,
            where_clause
        );
        let mut stmt = conn.prepare(&sql)?;
        let iter = stmt
            .query_map(params, |row| Ok(Self::try_from(row)?))?
            .collect::<rusqlite::Result<_>>()?;
        Ok(iter)
    }

    fn select_one(
        conn: &rusqlite::Connection,
        where_clause: &str,
        params: impl rusqlite::Params,
    ) -> rusqlite::Result<Option<Self>> {
        let sql = format!(
            "SELECT {} FROM {} {} LIMIT 1",
            Self::column_getters(),
            Self::TABLE_NAME,
            where_clause
        );
        let mut stmt = conn.prepare(&sql)?;
        let row = stmt
            .query_row(params, |row| Ok(Self::try_from(row)?))
            .optional()?;
        Ok(row)
    }

    fn delete(
        conn: &rusqlite::Connection,
        where_clause: &str,
        params: impl rusqlite::Params,
    ) -> rusqlite::Result<usize> {
        let sql = format!("DELETE FROM {} {}", Self::TABLE_NAME, where_clause);
        let mut stmt = conn.prepare(&sql).unwrap();
        stmt.execute(params)
    }

    fn drop_table(conn: &rusqlite::Connection) -> rusqlite::Result<usize> {
        let sql = format!("DROP TABLE IF EXISTS {}", Self::TABLE_NAME);
        conn.execute(&sql, ())
    }
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
