use crate::database;
use crate::database::objects::Kind;
use deno_core::error::AnyError;
use deno_core::op2;
use deno_core::v8::Number;
use deno_core::{OpState, serde_v8};
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
        Some(object) => Some(String::from_utf8(object.data)?),
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

#[op2(fast)]
pub fn db_set_number(
    state: &mut OpState,
    #[string] key: String,
    data: f64,
) -> Result<(), AnyError> {
    let core = state.borrow::<Arc<database::core::Core>>().clone();
    println!("{}", data);
    let data_bytes = data.to_le_bytes().to_vec();
    core.set(&key, Kind::Number, data_bytes);
    Ok(())
}
