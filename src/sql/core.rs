use base64;
use rusqlite::{Connection, Result, ToSql, params};
use serde_json::{Map, Value};
use std::collections::HashMap;

pub struct DbCore {
    conn: Connection,
}

impl DbCore {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(DbCore { conn })
    }

    pub fn create_table(&self, sql: &str) -> Result<()> {
        self.conn.execute(sql, [])?;
        Ok(())
    }

    pub fn insert(&self, table: &str, row: &HashMap<&str, &dyn ToSql>) -> Result<usize> {
        let columns: Vec<&str> = row.keys().cloned().collect();
        let values: Vec<&dyn ToSql> = columns.iter().map(|k| row[k]).collect();
        let placeholders: Vec<String> = (0..columns.len()).map(|_| "?".to_string()).collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            columns.join(", "),
            placeholders.join(", ")
        );

        self.conn.execute(&sql, values)
    }

    pub fn update(
        &self,
        table: &str,
        row: &HashMap<&str, &dyn ToSql>,
        condition: &str,
        condition_params: &[&dyn ToSql],
    ) -> Result<usize> {
        let columns: Vec<&str> = row.keys().cloned().collect();
        let values: Vec<&dyn ToSql> = columns.iter().map(|k| row[k]).collect();
        let assignments: Vec<String> = columns.iter().map(|k| format!("{} = ?", k)).collect();

        let sql = format!(
            "UPDATE {} SET {} WHERE {}",
            table,
            assignments.join(", "),
            condition
        );

        let mut all_params: Vec<&dyn ToSql> = Vec::new();
        all_params.extend(values);
        all_params.extend_from_slice(condition_params);

        self.conn.execute(&sql, all_params)
    }

    pub fn delete(
        &self,
        table: &str,
        condition: &str,
        condition_params: &[&dyn ToSql],
    ) -> Result<usize> {
        let sql = format!("DELETE FROM {} WHERE {}", table, condition);
        self.conn.execute(&sql, condition_params)
    }

    pub fn query_to_json(&self, sql: &str, params: &[&dyn ToSql]) -> Result<Value> {
        let mut stmt = self.conn.prepare(sql)?;
        let column_names = stmt.column_names().to_vec();

        let rows = stmt.query_map(params, |row| {
            let mut map = Map::new();
            for (i, col_name) in column_names.iter().enumerate() {
                let val: Result<Value, _> = match row.get_ref(i)? {
                    rusqlite::types::ValueRef::Null => Ok(Value::Null),
                    rusqlite::types::ValueRef::Integer(i) => Ok(Value::from(i)),
                    rusqlite::types::ValueRef::Real(f) => Ok(Value::from(f)),
                    rusqlite::types::ValueRef::Text(t) => {
                        Ok(Value::from(String::from_utf8_lossy(t)))
                    }
                    rusqlite::types::ValueRef::Blob(b) => Ok(Value::from(base64::encode(b))),
                };
                map.insert(col_name.to_string(), val?);
            }
            Ok(Value::Object(map))
        })?;

        let results: Result<Vec<Value>> = rows.collect();
        Ok(Value::from(results?))
    }
    pub fn list_tables(&self) -> Result<Value> {
        let mut stmt = self.conn.prepare(
            "SELECT name, sql FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        )?;

        let tables_iter = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let sql: String = row.get(1)?;
            Ok((name, sql))
        })?;

        let mut results = Vec::new();

        for table in tables_iter {
            let (name, sql) = table?;
            let count_sql = format!("SELECT COUNT(*) FROM {}", name);
            let count: i64 = self.conn.query_row(&count_sql, [], |row| row.get(0))?;

            results.push(json!({
                "name": name,
                "rows": count,
                "schema": sql
            }));
        }

        Ok(Value::from(results))
    }
    pub fn create_table_dynamic(
        &self,
        table_name: &str,
        columns: &[(impl AsRef<str>, impl AsRef<str>)],
        primary_key: Option<&str>,
    ) -> Result<()> {
        let mut col_defs: Vec<String> = columns
            .iter()
            .map(|(name, typ)| format!("{} {}", name.as_ref(), typ.as_ref()))
            .collect();

        if let Some(pk) = primary_key {
            col_defs.push(format!("PRIMARY KEY ({})", pk));
        }

        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            table_name,
            col_defs.join(", ")
        );

        self.conn.execute(&sql, [])?;
        Ok(())
    }
    pub fn create_table_with_access_rw(
        &self,
        table_name: &str,
        columns: &[(impl AsRef<str>, impl AsRef<str>)],
        primary_key: Option<&str>,
        extra_rules: &[&str],
    ) -> Result<()> {
        let mut col_defs: Vec<String> = columns
            .iter()
            .map(|(name, typ)| format!("{} {}", name.as_ref(), typ.as_ref()))
            .collect();

        col_defs.push("r INTEGER NOT NULL".to_string());
        col_defs.push("rw INTEGER NOT NULL".to_string());

        if let Some(pk) = primary_key {
            col_defs.push(format!("PRIMARY KEY ({})", pk));
        }

        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            table_name,
            col_defs.join(", ")
        );

        self.conn.execute(&sql, [])?;

        for rule in extra_rules {
            self.conn.execute(rule, [])?;
        }

        Ok(())
    }
    pub fn query_with_read_access(
        &self,
        base_sql: &str,
        user: &UserContext,
        params: &[&dyn ToSql],
    ) -> Result<Value> {
        let filtered_sql = if base_sql.to_lowercase().contains("where") {
            format!("{} AND r <= {}", base_sql, user.r)
        } else {
            format!("{} WHERE r <= {}", base_sql, user.r)
        };
        self.query_to_json(&filtered_sql, params)
    }
    pub fn execute_with_write_access(
        &self,
        sql: &str,
        mut params: Vec<&dyn ToSql>,
        user: &UserContext,
    ) -> Result<usize> {
        params.push(&user.rw);
        self.conn.execute(sql, params)
    }
    pub fn create_table_with_access_rwx(
        &self,
        table_name: &str,
        columns: &[(impl AsRef<str>, impl AsRef<str>)],
        primary_key: Option<&str>,
        extra_rules: &[&str],
    ) -> Result<()> {
        let mut col_defs: Vec<String> = columns
            .iter()
            .map(|(name, typ)| format!("{} {}", name.as_ref(), typ.as_ref()))
            .collect();

        col_defs.push("r INTEGER NOT NULL".to_string());
        col_defs.push("rw INTEGER NOT NULL".to_string());
        col_defs.push("rwx INTEGER NOT NULL".to_string());

        if let Some(pk) = primary_key {
            col_defs.push(format!("PRIMARY KEY ({})", pk));
        }

        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            table_name,
            col_defs.join(", ")
        );

        self.conn.execute(&sql, [])?;

        for rule in extra_rules {
            self.conn.execute(rule, [])?;
        }

        Ok(())
    }
}
