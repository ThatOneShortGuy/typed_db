use std::hint::black_box;

use chrono::prelude::*;
use criterion::{Criterion, criterion_group, criterion_main};
use typed_db::prelude::*;

#[derive(Debug, Clone, DbTable)]
pub struct UsersTable {
    #[primary_key]
    pub id: u32,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    #[default(CURRENT_TIMESTAMP)]
    pub created_date: DateTime<Utc>,
}

fn create_tables(c: &mut Criterion) {
    // const DB_NAME: &str = "test.db";
    const DB_NAME: &str = ":memory:";
    let db_path = std::path::PathBuf::from(DB_NAME);

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let _ = conn.execute("PRAGMA journal_mode = WAL;", []);
    UsersTable::create_table(&conn).unwrap();

    let a = UsersTable::new()
        .with_name("Alice")
        .with_email("alice@example.com")
        .with_password_hash("1234567890");

    c.bench_function("Insert build_raw", |b| {
        b.iter(|| {
            black_box(a.clone()).build_raw(&conn).unwrap();
        })
    });

    c.bench_function("Insert build", |b| {
        b.iter(|| {
            black_box(a.clone()).build(&conn).unwrap();
        })
    });

    c.bench_function("Insert build_val", |b| {
        b.iter(|| {
            black_box(a.clone()).build_val(&conn).unwrap();
        })
    });
    conn.close().unwrap();
    // Delete the db file.
    if db_path.exists() {
        std::fs::remove_file(&db_path).unwrap();
    }
}

criterion_group!(benches, create_tables);
criterion_main!(benches);
