use deno_core::error::AnyError;
use deno_core::op2;

#[op2]
#[string]
pub fn op_utils_uuidV4() -> Result<String, AnyError> {
    let uuid_bytes = uuid::Uuid::new_v4();
    Ok(uuid_bytes.to_string())
}
