use crate::database;
use crate::database::objects::Kind;
use deno_core::OpState;
use deno_core::error::AnyError;
use deno_core::op2;
use std::sync::Arc;

#[op2]
#[string]
pub fn db_get_value(
    state: &mut OpState,
    #[string] key: String,
) -> Result<Option<String>, AnyError> {
    let core = state.borrow::<Arc<database::core::Core>>().clone();
    let value = core.get(&key);

    Ok(match value {
        Some(bytes) => Some(String::from_utf8(bytes)?),
        None => None,
    })
}

#[op2(fast)]
pub fn db_set_string(
    state: &mut OpState,
    #[string] key: String,
    #[string] data: String,
) -> Result<(), AnyError> {
    let core = state.borrow::<Arc<database::core::Core>>().clone();

    let data_bytes = data.into_bytes();
    core.set(&key, Kind::String, data_bytes);
    Ok(())
}
