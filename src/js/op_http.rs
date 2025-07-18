use deno_core::error::AnyError;
use deno_core::op2;

use reqwest::Client;

#[op2(async)]
#[string]
pub async fn op_http_get(#[string] url: String) -> Result<String, AnyError> {
    let body = Client::new().get(&url).send().await?.text().await?;
    Ok(body)
}

#[op2(async)]
#[string]
pub async fn op_http_post(
    #[string] url: String,
    #[string] body: String,
) -> Result<String, AnyError> {
    let body = Client::new()
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?
        .text()
        .await?;
    Ok(body)
}

#[op2(async)]
#[string]
pub async fn op_http_put(
    #[string] url: String,
    #[string] body: String,
) -> Result<String, AnyError> {
    let body = Client::new()
        .put(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?
        .text()
        .await?;
    Ok(body)
}

#[op2(async)]
#[string]
pub async fn op_http_delete(#[string] url: String) -> Result<String, AnyError> {
    let body = Client::new().delete(&url).send().await?.text().await?;
    Ok(body)
}

// Opcjonalnie PATCH:
#[op2(async)]
#[string]
pub async fn op_http_patch(
    #[string] url: String,
    #[string] body: String,
) -> Result<String, AnyError> {
    let body = Client::new()
        .patch(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?
        .text()
        .await?;
    Ok(body)
}
