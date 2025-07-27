use rusqlite::Row;
use serde::{Deserialize, Serialize};

pub trait DbModel: Serialize + for<'de> Deserialize<'de> {
    fn table_name() -> &'static str;
    fn from_row(row: &Row) -> rusqlite::Result<Self>
    where
        Self: Sized;
}
