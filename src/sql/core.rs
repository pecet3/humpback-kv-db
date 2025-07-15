use rusqlite::{Connection, Result, params};

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

    pub fn insert(&self, sql: &str, values: &[&dyn rusqlite::ToSql]) -> Result<usize> {
        self.conn.execute(sql, values)
    }

    pub fn delete(&self, sql: &str, values: &[&dyn rusqlite::ToSql]) -> Result<usize> {
        self.conn.execute(sql, values)
    }

    pub fn update(&self, sql: &str, values: &[&dyn rusqlite::ToSql]) -> Result<usize> {
        self.conn.execute(sql, values)
    }
}
