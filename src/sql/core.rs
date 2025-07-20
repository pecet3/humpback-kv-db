use std::sync::Mutex;

use deno_core::serde_json::{self, Map, Value};
use rusqlite::{Connection, Result, Row};

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    pub fn execute_batch(&self, sql: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(sql)
    }
    pub fn get_table_info(&self, table_name: &str) -> Result<serde_json::Value, rusqlite::Error> {
        let sql = format!("PRAGMA table_info({})", table_name);
        self.query_json(&sql)
    }

    pub fn list_tables(&self) -> Result<serde_json::Value, rusqlite::Error> {
        let sql = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name";
        self.query_json(sql)
    }
    pub fn query_json(&self, sql: &str) -> Result<Value> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(sql)?;
        let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let rows = stmt
            .query_map([], |row| Self::row_to_json(row, &column_names))?
            .collect::<Result<Vec<_>>>()?;

        Ok(Value::Array(rows))
    }
    fn row_to_json(row: &Row, column_names: &[String]) -> Result<Value> {
        let mut map = Map::new();

        for (i, col_name) in column_names.iter().enumerate() {
            let value: rusqlite::types::Value = row.get(i)?;

            let json_value = match value {
                rusqlite::types::Value::Null => Value::Null,
                rusqlite::types::Value::Integer(i) => Value::from(i),
                rusqlite::types::Value::Real(f) => Value::from(f),
                rusqlite::types::Value::Text(s) => Value::from(s),
                rusqlite::types::Value::Blob(b) => Value::from(base64::encode(b)),
            };

            map.insert(col_name.clone(), json_value);
        }

        Ok(Value::Object(map))
    }
}
