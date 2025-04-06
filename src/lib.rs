mod traits;
mod types;

use std::num::ParseIntError;

use chrono::prelude::*;
use rusqlite::{Connection, Params, ToSql, params, types::FromSql};
pub use traits::*;
use typed_db_derive::CommonTableExpression;
pub use typed_db_derive::DbTable;

pub mod prelude {
    pub use crate::traits::*;
    pub use typed_db_derive::DbTable;
}

type Id = i32;

#[derive(Debug, Clone, DbTable)]
pub struct User {
    #[primary_key]
    pub id: Id,
    pub name: String,
    #[unique]
    pub email: String,
    #[default(CURRENT_TIMESTAMP)]
    pub created_date: DateTime<Utc>,
}

#[derive(Debug, Clone, DbTable)]
pub struct UserRole {
    #[primary_key]
    pub id: Id,
    #[foreign_key(User::id)]
    pub user_id: Id,
    pub role: String,
    #[default(CURRENT_TIMESTAMP)]
    pub active_date: DateTime<Utc>,
}

#[derive(Debug, Clone, DbTable)]
pub struct UserTeam {
    #[primary_key]
    pub id: Id,
    #[foreign_key(User::id, on_delete = CASCADE)]
    pub team_member: Id,
    #[foreign_key(User::id, on_delete = CASCADE)]
    pub team_leader: Id,
    #[default(CURRENT_TIMESTAMP)]
    pub active_date: DateTime<Utc>,
}

#[derive(Debug, Clone, CommonTableExpression)]
#[cte_params("effective_time", "user_id")]
struct ActiveUser {
    #[param(User::id as "u", "u.id = params.user_id")]
    pub id: Id,
    #[param(User::name as "u", "u.id = params.user_id")]
    pub name: String,
    #[param(User::email as "u", "u.id = params.user_id")]
    pub email: String,
    #[param(
        UserRole::role as "ur",
        "ur.user_id = params.user_id
         AND ur.active_date <= params.effective_time
         ORDER BY ur.active_date DESC
         LIMIT 1"
    )]
    pub role: Option<String>,
    #[param(
        UserTeam::team_leader as "ut",
        "ut.team_member = params.user_id
         AND ut.active_date <= params.effective_time
         ORDER BY ut.active_date DESC
         LIMIT 1"
    )]
    pub team_leader: Option<Id>,
}

#[test]
fn t() -> Result<(), Box<dyn std::error::Error>> {
    let conn = rusqlite::Connection::open(":memory:")?;
    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    User::create_table(&conn)?;
    UserRole::create_table(&conn)?;
    UserTeam::create_table(&conn)?;

    let u1 = User::new()
        .with_name("Bob")
        .with_email("bob@example.com")
        .build_val(&conn)?;
    let u2 = User::new()
        .with_name("Alice")
        .with_email("alice@example.com")
        .build_val(&conn)?;
    UserRole::new()
        .with_user_id(u1.id)
        .with_role("Admin")
        .build_raw(&conn)?;
    UserTeam::new()
        .with_team_member(u1.id)
        .with_team_leader(u2.id)
        .build(&conn)?;

    let a = ActiveUser::select(&conn, params![Utc::now(), u1.id])?;

    // println!("{}", User::create_table_str());
    // println!("{}", UserRole::create_table_str());
    // println!("{}", UserTeam::create_table_str());
    // println!("{}", ActiveUser::cte_str());
    ActiveUser::print_query_plan(&conn, params![Utc::now(), u1.id])?;
    panic!("{a:#?}");

    Ok(())
}
