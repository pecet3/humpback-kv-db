use std::time::{SystemTime, UNIX_EPOCH};
pub fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos() as i64
}
