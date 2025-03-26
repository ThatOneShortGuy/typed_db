mod traits;
mod types;

use chrono::prelude::*;
pub use traits::*;
pub use typed_db_derive::DbTable;

pub mod prelude {
    pub use crate::traits::*;
    pub use typed_db_derive::DbTable;
}

#[derive(Debug, Clone, DbTable)]
pub struct UsersTable {
    #[primary_key]
    pub id: u32,
    pub name: String,
    #[unique]
    pub email: String,
    pub password_hash: String,
    #[default(CURRENT_TIMESTAMP)]
    pub created_date: DateTime<Utc>,
}

#[derive(Debug, Clone, DbTable)]
pub struct ParentTable {
    #[composite_key]
    #[foreign_key(UsersTable::id, on_delete = CASCADE, on_update = SET NULL)]
    pub user_id: u32,
    #[composite_key]
    #[foreign_key(UsersTable::id)]
    pub parent_id: u32,
    #[default(CURRENT_TIMESTAMP)]
    pub created_date: DateTime<Utc>,
}
