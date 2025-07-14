use deno_core::error::AnyError;
use deno_core::op2;

#[op2(async)]
#[string]
pub async fn op_http_get(#[string] url: String) -> Result<String, AnyError> {
    let body = reqwest::get(url).await?.text().await?;
    Ok(body)
}
