use rusqlite::{Connection, Result, Row, params};
use std::{path::Path, str::FromStr};

use crate::object_service::{self, Kind};

#[derive(Debug, Clone)]
pub struct ObjectDescriptor {
    pub data: Vec<u8>,
    pub created_at: u128,
    pub updated_at: u128,
    pub last_opened_at: u128,
    pub kind: object_service::Kind,
    pub offset: usize,
    pub size: usize,
    pub key: String,
}

impl ObjectDescriptor {
    pub fn new(key: String, data: Vec<u8>, kind: Kind) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        Self {
            size: data.len(),
            data,
            created_at: now,
            updated_at: now,
            last_opened_at: now,
            kind,
            offset: 0,
            key,
        }
    }
}

pub struct StoreDb {
    conn: Connection,
}

impl StoreDb {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.create_table_if_not_exists()?;
        Ok(db)
    }

    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.create_table_if_not_exists()?;
        Ok(db)
    }

    fn create_table_if_not_exists(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS object_descriptors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT UNIQUE NOT NULL,
                data BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                last_opened_at INTEGER NOT NULL,
                kind TEXT NOT NULL,
                offset_val INTEGER NOT NULL,
                size INTEGER NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn create(&self, obj: &ObjectDescriptor) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO object_descriptors 
            (key, data, created_at, updated_at, last_opened_at, kind, offset_val, size)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                obj.key,
                obj.data,
                obj.created_at as i64,
                obj.updated_at as i64,
                obj.last_opened_at as i64,
                obj.kind.to_string(),
                obj.offset as i64,
                obj.size as i64
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn read_by_key(&self, key: &str) -> Result<Option<ObjectDescriptor>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, data, created_at, updated_at, last_opened_at, kind, offset_val, size
             FROM object_descriptors WHERE key = ?1",
        )?;

        let obj_iter = stmt.query_map([key], |row| self.row_to_object_descriptor(row))?;

        for obj in obj_iter {
            return Ok(Some(obj?));
        }

        Ok(None)
    }

    pub fn read_by_id(&self, id: i64) -> Result<Option<ObjectDescriptor>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, data, created_at, updated_at, last_opened_at, kind, offset_val, size
             FROM object_descriptors WHERE id = ?1",
        )?;

        let obj_iter = stmt.query_map([id], |row| self.row_to_object_descriptor(row))?;

        for obj in obj_iter {
            return Ok(Some(obj?));
        }

        Ok(None)
    }

    pub fn read_all(&self) -> Result<Vec<ObjectDescriptor>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, data, created_at, updated_at, last_opened_at, kind, offset_val, size
             FROM object_descriptors",
        )?;

        let obj_iter = stmt.query_map([], |row| self.row_to_object_descriptor(row))?;

        let mut objects = Vec::new();
        for obj in obj_iter {
            objects.push(obj?);
        }

        Ok(objects)
    }

    pub fn update(&self, key: &str, obj: &ObjectDescriptor) -> Result<usize> {
        let updated_rows = self.conn.execute(
            "UPDATE object_descriptors SET 
            key = ?1, data = ?2, created_at = ?3, updated_at = ?4, last_opened_at = ?5, 
            kind = ?6, offset_val = ?7, size = ?8
            WHERE key = ?9",
            params![
                obj.key,
                obj.data,
                obj.created_at as i64,
                obj.updated_at as i64,
                obj.last_opened_at as i64,
                obj.kind.to_string(),
                obj.offset as i64,
                obj.size as i64,
                key
            ],
        )?;

        Ok(updated_rows)
    }

    pub fn update_last_opened(&self, key: &str) -> Result<usize> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        let updated_rows = self.conn.execute(
            "UPDATE object_descriptors SET last_opened_at = ?1 WHERE key = ?2",
            params![now, key],
        )?;

        Ok(updated_rows)
    }

    pub fn delete_by_key(&self, key: &str) -> Result<usize> {
        let deleted_rows = self
            .conn
            .execute("DELETE FROM object_descriptors WHERE key = ?1", [key])?;

        Ok(deleted_rows)
    }

    pub fn delete_by_id(&self, id: i64) -> Result<usize> {
        let deleted_rows = self
            .conn
            .execute("DELETE FROM object_descriptors WHERE id = ?1", [id])?;

        Ok(deleted_rows)
    }

    pub fn delete_all(&self) -> Result<usize> {
        let deleted_rows = self.conn.execute("DELETE FROM object_descriptors", [])?;
        Ok(deleted_rows)
    }

    fn row_to_object_descriptor(&self, row: &Row) -> Result<ObjectDescriptor> {
        let kind_str: String = row.get(5)?;
        let kind = Kind::from_str(&kind_str).unwrap();

        Ok(ObjectDescriptor {
            key: row.get(0)?,
            data: row.get(1)?,
            created_at: row.get::<_, i64>(2)? as u128,
            updated_at: row.get::<_, i64>(3)? as u128,
            last_opened_at: row.get::<_, i64>(4)? as u128,
            kind,
            offset: row.get::<_, i64>(6)? as usize,
            size: row.get::<_, i64>(7)? as usize,
        })
    }

    pub fn exists(&self, key: &str) -> Result<bool> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM object_descriptors WHERE key = ?1")?;
        let exists = stmt.exists([key])?;
        Ok(exists)
    }

    pub fn count(&self) -> Result<i64> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM object_descriptors", [], |row| {
                    row.get(0)
                })?;
        Ok(count)
    }

    pub fn get_keys(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT key FROM object_descriptors")?;
        let key_iter = stmt.query_map([], |row| Ok(row.get::<_, String>(0)?))?;

        let mut keys = Vec::new();
        for key in key_iter {
            keys.push(key?);
        }

        Ok(keys)
    }
}
