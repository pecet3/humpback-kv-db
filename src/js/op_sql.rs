use crate::kv::objects::Kind;
use crate::{kv, sql};
use deno_core::OpState;
use deno_core::error::AnyError;
use deno_core::op2;
use deno_core::serde_json::{self, json};
use std::sync::Arc;

#[op2]
#[serde]
pub fn op_sql_query(
    state: &mut OpState,
    #[string] query: String,
) -> Result<serde_json::Value, AnyError> {
    let db: &sql::core::Db = state.borrow::<sql::core::Db>();
    let value = db.query_json(&query).unwrap();
    Ok(value)
}
#[op2(fast)]
pub fn op_sql_exec(state: &mut OpState, #[string] statement: String) -> Result<(), AnyError> {
    let db: &sql::core::Db = state.borrow::<sql::core::Db>();
    db.execute_batch(&statement)?;
    Ok(())
}
