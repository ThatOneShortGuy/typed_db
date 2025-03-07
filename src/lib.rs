mod traits;
mod types;

use chrono::prelude::*;
pub use traits::*;
use typed_db_derive::DbTable;

#[derive(Debug, Clone, DbTable)]
pub struct UsersTable {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub created_date: DateTime<Utc>,
}
