use deno_core::error::AnyError;
use deno_core::op2;
#[op2(async)]
#[string]
pub async fn file_read(#[string] path: String) -> Result<String, AnyError> {
    let contents = tokio::fs::read_to_string(path).await?;
    Ok(contents)
}

#[op2(async)]
pub async fn file_write(
    #[string] path: String,
    #[string] contents: String,
) -> Result<(), AnyError> {
    tokio::fs::write(path, contents).await?;
    Ok(())
}

#[op2(fast)]
pub fn file_remove(#[string] path: String) -> Result<(), AnyError> {
    std::fs::remove_file(path)?;
    Ok(())
}
