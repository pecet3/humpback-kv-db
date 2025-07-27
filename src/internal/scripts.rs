use crate::internal::{db::InternalDb, helpers::now_ns};
use rusqlite::{Result, params};

#[derive(Debug)]
pub struct Script {
    pub id: i32,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug)]
pub struct ScriptSnapshot {
    pub id: i32,
    pub script_id: i32,
    pub content: String,
    pub created_at: i64,
}

impl InternalDb {
    pub fn create_script(&self, name: &str) -> Result<i32> {
        let created_at = now_ns();
        let conn = self.db.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO scripts (name, created_at) VALUES (?1, ?2)",
            params![name, created_at],
        )?;
        Ok(conn.last_insert_rowid() as i32)
    }

    pub fn add_script_snapshot(&self, script_id: i32, content: &str) -> Result<i32> {
        let created_at = now_ns();
        let conn = self.db.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO script_snapshots (script_id, content, created_at) VALUES (?1, ?2, ?3)",
            params![script_id, content, created_at],
        )?;
        Ok(conn.last_insert_rowid() as i32)
    }

    pub fn get_last_script_snapshot(&self, script_id: i32) -> Result<Option<ScriptSnapshot>> {
        let conn = self.db.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, script_id, content, created_at
             FROM script_snapshots
             WHERE script_id = ?1
             ORDER BY created_at DESC
             LIMIT 1",
        )?;

        let mut rows = stmt.query(params![script_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(ScriptSnapshot {
                id: row.get(0)?,
                script_id: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_scripts(&self) -> Result<Vec<Script>> {
        let conn = self.db.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, created_at FROM scripts ORDER BY id ASC")?;

        let scripts = stmt.query_map([], |row| {
            Ok(Script {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;

        Ok(scripts.map(|s| s.unwrap()).collect())
    }

    pub fn get_script_by_name(&self, name: &str) -> Result<Option<Script>> {
        let conn = self.db.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, created_at FROM scripts WHERE name = ?1")?;

        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Script {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            }))
        } else {
            Ok(None)
        }
    }
}
