use crate::{
    STORE_PATH,
    http_server::router::{AUTH_TOKEN, AppState},
    js::{self, event::Event, runtime::Runtime},
    kv::{self, core::Core, objects::Kind},
    sql::{self, core::Db},
};
use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    response::{Html, Json as ResponseJson},
    routing::{get, post},
};
use deno_core::serde_json::{self, json};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, io::Write, str::FromStr, sync::Arc, time::Duration};
use tokio::time::timeout;

fn verify_token(token: &str) -> bool {
    token == AUTH_TOKEN
}

pub async fn serve_html() -> Html<&'static str> {
    Html(include_str!("../../index.html"))
}

fn create_error_response(error: &str) -> (StatusCode, ResponseJson<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        ResponseJson(ErrorResponse {
            status: "error".to_string(),
            error: error.to_string(),
        }),
    )
}

fn create_success_response(data: Option<serde_json::Value>) -> ResponseJson<SuccessResponse> {
    ResponseJson(SuccessResponse {
        status: "success".to_string(),
        data,
    })
}

#[derive(Deserialize)]
pub struct BaseRequest {
    token: String,
}

#[derive(Deserialize)]
pub struct GetRequest {
    token: String,
    key: String,
}

#[derive(Deserialize)]
pub struct SetRequest {
    token: String,
    key: String,
    kind: String,
    data: String,
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    token: String,
    key: String,
}

#[derive(Deserialize)]
pub struct ListRequest {
    token: String,
}

#[derive(Deserialize)]
pub struct ListTypeRequest {
    token: String,
    kind: String,
}

#[derive(Deserialize)]
pub struct ExecRequest {
    token: String,
    key: String,
}
#[derive(Deserialize)]
pub struct ExecNowRequest {
    token: String,
    code: String,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    status: String,
    data: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    status: String,
    error: String,
}

#[derive(Serialize)]
pub struct ListItem {
    key: String,
    kind: String,
    size: usize,
}

#[derive(Deserialize)]
pub struct SqlExecRequest {
    token: String,
    statement: String,
}
#[derive(Deserialize)]
pub struct SqlQueryRequest {
    token: String,
    query: String,
}

type ApiResult<T> = Result<ResponseJson<T>, (StatusCode, ResponseJson<ErrorResponse>)>;
pub async fn handle_get(
    State(state): State<AppState>,
    Json(request): Json<GetRequest>,
) -> ApiResult<SuccessResponse> {
    if !verify_token(&request.token) {
        return Err(create_error_response("Invalid token"));
    }

    let start = std::time::Instant::now();
    let object = state.core.get_async(&request.key).await;
    let duration = start.elapsed();
    println!("GET completed in {:.2?}", duration);

    match object {
        Some(object) => {
            let data = match object.desc.kind {
                Kind::Number => {
                    let mut arr = [0u8; 8];
                    arr.copy_from_slice(&object.data[..8]);
                    let number = f64::from_le_bytes(arr);
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(number).unwrap_or(serde_json::Number::from(0)),
                    )
                }
                Kind::Boolean => {
                    let value = object.data[0] != 0;
                    serde_json::Value::Bool(value)
                }
                Kind::Json => {
                    let json_str = String::from_utf8_lossy(&object.data);
                    serde_json::from_str(&json_str)
                        .unwrap_or(serde_json::Value::String(json_str.to_string()))
                }
                _ => {
                    let string_data = String::from_utf8_lossy(&object.data);
                    serde_json::Value::String(string_data.to_string())
                }
            };

            Ok(create_success_response(Some(data)))
        }
        None => Err(create_error_response("Not found")),
    }
}

pub async fn handle_set(
    State(state): State<AppState>,
    Json(request): Json<SetRequest>,
) -> ApiResult<SuccessResponse> {
    if !verify_token(&request.token) {
        return Err(create_error_response("Invalid token"));
    }

    if request.key.len() > 256 {
        return Err(create_error_response(
            "Key is too long. Max key length - 256 bytes",
        ));
    }

    let kind = match Kind::from_str(&request.kind) {
        Ok(k) => k,
        Err(_) => return Err(create_error_response("Unknown kind")),
    };

    let data_buf = match kind {
        Kind::Number => {
            let number: f64 = request
                .data
                .parse()
                .map_err(|_| create_error_response("Invalid number format"))?;
            number.to_le_bytes().to_vec()
        }
        Kind::Boolean => {
            let boolean: bool = request
                .data
                .parse()
                .map_err(|_| create_error_response("Invalid boolean format"))?;
            vec![if boolean { 1 } else { 0 }]
        }
        _ => request.data.into_bytes(),
    };

    let start = std::time::Instant::now();
    state
        .core
        .set_async(&request.key, kind, data_buf.clone())
        .await;
    let duration = start.elapsed();
    println!(
        "SET completed in {:.2?} ({} bytes)",
        duration,
        data_buf.len()
    );

    Ok(create_success_response(None))
}

pub async fn handle_delete(
    State(state): State<AppState>,
    Json(request): Json<DeleteRequest>,
) -> ApiResult<SuccessResponse> {
    if !verify_token(&request.token) {
        return Err(create_error_response("Invalid token"));
    }

    let start = std::time::Instant::now();
    match state.core.delete_soft(&request.key).await {
        Ok(_) => {
            let duration = start.elapsed();
            println!("DELETE completed in {:.2?}", duration);
            Ok(create_success_response(None))
        }
        Err(_) => Err(create_error_response("Not found")),
    }
}

pub async fn handle_list(
    State(state): State<AppState>,
    Json(request): Json<ListRequest>,
) -> ApiResult<SuccessResponse> {
    if !verify_token(&request.token) {
        return Err(create_error_response("Invalid token"));
    }

    let start = std::time::Instant::now();
    match state.core.list().await {
        Ok(list) => {
            let items: Vec<ListItem> = list
                .iter()
                .map(|element| ListItem {
                    key: element.key.clone(),
                    kind: element.kind.to_string(),
                    size: element.size as usize,
                })
                .collect();

            let duration = start.elapsed();
            println!("LIST completed in {:.2?}", duration);

            Ok(create_success_response(Some(
                serde_json::to_value(items).unwrap_or(serde_json::Value::Array(vec![])),
            )))
        }
        Err(_) => Err(create_error_response("Unable to list objects")),
    }
}

pub async fn handle_list_type(
    State(state): State<AppState>,
    Json(request): Json<ListTypeRequest>,
) -> ApiResult<SuccessResponse> {
    if !verify_token(&request.token) {
        return Err(create_error_response("Invalid token"));
    }

    let kind_enum = match Kind::from_str(&request.kind) {
        Ok(k) => k,
        Err(_) => return Err(create_error_response("Invalid type")),
    };

    let start = std::time::Instant::now();
    match state.core.list_by_kind(kind_enum).await {
        Ok(list) => {
            let items: Vec<ListItem> = list
                .iter()
                .map(|element| ListItem {
                    key: element.key.clone(),
                    kind: element.kind.to_string(),
                    size: element.size as usize,
                })
                .collect();

            let duration = start.elapsed();
            println!("LIST_TYPE completed in {:.2?}", duration);

            Ok(create_success_response(Some(
                serde_json::to_value(items).unwrap_or(serde_json::Value::Array(vec![])),
            )))
        }
        Err(_) => Err(create_error_response("Unable to list objects")),
    }
}

pub async fn handle_exec(
    State(state): State<AppState>,
    Json(request): Json<ExecRequest>,
) -> ApiResult<SuccessResponse> {
    if !verify_token(&request.token) {
        return Err(create_error_response("Invalid token"));
    }

    let object = state.core.get_async(&request.key).await;
    match object {
        Some(object) => match String::from_utf8(object.data) {
            Ok(code) => {
                let event = js::event::Event::new_code_event(code);
                state.runtime.push_event(event);
                Ok(create_success_response(None))
            }
            Err(_) => Err(create_error_response("Invalid UTF-8")),
        },
        None => Err(create_error_response("Not found")),
    }
}

pub async fn handle_exec_now(
    State(state): State<AppState>,
    Json(request): Json<ExecNowRequest>,
) -> ApiResult<SuccessResponse> {
    if !verify_token(&request.token) {
        return Err(create_error_response("Invalid token"));
    }
    let event = js::event::Event::new_code_event(request.code);
    let rx = state.runtime.push_event(event);
    if let Ok(Ok(mut response)) = timeout(Duration::from_secs(5), rx).await {
        let resp = response.take();
        return Ok(create_success_response(Some(resp)));
    }
    Ok(create_success_response(None))
}

pub async fn handle_sql_exec(
    State(state): State<AppState>,
    Json(request): Json<SqlExecRequest>,
) -> ApiResult<SuccessResponse> {
    if !verify_token(&request.token) {
        return Err(create_error_response("Invalid token"));
    }

    match state.db.execute_batch(&request.statement) {
        Ok(()) => {
            let response_data = serde_json::json!({
                "message": "Statement executed successfully",
                "rows_affected": null
            });
            Ok(create_success_response(Some(response_data)))
        }
        Err(e) => Err(create_error_response(&format!(
            "SQL execution error: {}",
            e
        ))),
    }
}

pub async fn handle_sql_query(
    State(state): State<AppState>,
    Json(request): Json<SqlQueryRequest>,
) -> ApiResult<SuccessResponse> {
    if !verify_token(&request.token) {
        return Err(create_error_response("Invalid token"));
    }

    match state.db.query_json(&request.query) {
        Ok(data) => {
            let rows_count = match &data {
                serde_json::Value::Array(arr) => arr.len(),
                _ => 0,
            };

            let response_data = serde_json::json!({
                "data": data,
                "rows_count": rows_count
            });

            Ok(create_success_response(Some(response_data)))
        }
        Err(e) => Err(create_error_response(&format!("SQL query error: {}", e))),
    }
}
