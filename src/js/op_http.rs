use deno_core::error::AnyError;
use deno_core::op2;

use std::io::Read;

#[op2]
#[string]
pub fn op_http_get(#[string] url: String) -> Result<String, AnyError> {
    let response = ureq::get(&url).call()?;
    let body = response.into_body().read_to_string().unwrap();
    Ok(body)
}

#[op2]
#[string]
pub fn op_http_post(#[string] url: String, #[string] body: String) -> Result<String, AnyError> {
    let response = ureq::post(&url)
        .header("Content-Type", "application/json")
        .send(body)
        .unwrap();
    let body = response.into_body().read_to_string().unwrap();
    Ok(body)
}

#[op2]
#[string]
pub fn op_http_put(#[string] url: String, #[string] body: String) -> Result<String, AnyError> {
    let response = ureq::put(&url)
        .header("Content-Type", "application/json")
        .send(&body)?;
    let body = response.into_body().read_to_string().unwrap();
    Ok(body)
}

#[op2]
#[string]
pub fn op_http_delete(#[string] url: String) -> Result<String, AnyError> {
    let response = ureq::delete(&url).call()?;
    let body = response.into_body().read_to_string().unwrap();
    Ok(body)
}

// #[op2]
// #[string]
// pub fn op_http_patch(#[string] url: String, #[string] body: String) -> Result<String, AnyError> {
//     let response = ureq::patch(&url)
//         .set("Content-Type", "application/json")
//         .send_string(&body)?;
//     let mut reader = response.into_reader();
//     let mut resp_body = String::new();
//     reader.read_to_string(&mut resp_body)?;
//     Ok(resp_body)
// }
