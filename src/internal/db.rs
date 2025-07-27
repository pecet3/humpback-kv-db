use std::sync::Mutex;

use crate::{INTERNAL_STORE_PATH, sql};
use rusqlite::{Connection, Result};

pub struct InternalDb {
    pub db: sql::db::Db,
}
fn init_schema(conn: &Mutex<Connection>) -> Result<(), rusqlite::Error> {
    let conn = conn.lock().unwrap();
    conn.execute_batch(
        "
            CREATE TABLE IF NOT EXISTS scripts (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL UNIQUE,
                created_at  INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS script_snapshots (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                script_id   INTEGER NOT NULL,
                content     TEXT NOT NULL,
                created_at  INTEGER NOT NULL,
                FOREIGN KEY(script_id) REFERENCES scripts(id) ON DELETE CASCADE
            );
            ",
    )?;
    Ok(())
}
impl InternalDb {
    pub fn new() -> Result<Self, rusqlite::Error> {
        let db = sql::db::Db::new(INTERNAL_STORE_PATH)?;
        init_schema(&db.conn)?;
        Ok(InternalDb { db })
    }
}
