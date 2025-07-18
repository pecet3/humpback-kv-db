use crate::{
    js::{self, event::Event, runtime::Runtime},
    kv::{self, core::Core, objects::Kind},
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
use tokio::{signal, sync::Notify, time::timeout};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

const AUTH_TOKEN: &str = "humpback_secret_token_2024";

#[derive(Clone)]
pub struct AppState {
    pub core: Arc<Core>,
    pub runtime: Arc<Runtime>,
}

#[derive(Deserialize)]
struct BaseRequest {
    token: String,
}

#[derive(Deserialize)]
struct GetRequest {
    token: String,
    key: String,
}

#[derive(Deserialize)]
struct SetRequest {
    token: String,
    key: String,
    kind: String,
    data: String,
}

#[derive(Deserialize)]
struct DeleteRequest {
    token: String,
    key: String,
}

#[derive(Deserialize)]
struct ListRequest {
    token: String,
}

#[derive(Deserialize)]
struct ListTypeRequest {
    token: String,
    kind: String,
}

#[derive(Deserialize)]
struct ExecRequest {
    token: String,
    key: String,
}
#[derive(Deserialize)]
struct ExecNowRequest {
    token: String,
    code: String,
}

#[derive(Serialize)]
struct SuccessResponse {
    status: String,
    data: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct ErrorResponse {
    status: String,
    error: String,
}

#[derive(Serialize)]
struct ListItem {
    key: String,
    kind: String,
    size: usize,
}

type ApiResult<T> = Result<ResponseJson<T>, (StatusCode, ResponseJson<ErrorResponse>)>;

#[tokio::main]
pub async fn run() -> Result<(), Box<dyn Error>> {
    let core = kv::core::Core::new().unwrap();
    let runtime: Arc<Runtime> = js::runtime::Runtime::new(Arc::clone(&core));
    let state = AppState {
        core: Arc::clone(&core),
        runtime,
    };

    let app = Router::new()
        .route("/", get(serve_html))
        .route("/get", post(handle_get))
        .route("/set", post(handle_set))
        .route("/delete", post(handle_delete))
        .route("/list", post(handle_list))
        .route("/listType", post(handle_list_type))
        .route("/exec", post(handle_exec))
        .route("/execNow", post(handle_exec_now))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    println!("Humpback KV Database HTTP Server is listening on 127.0.0.1:8080");

    // Graceful shutdown
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        println!("Exit signal received\nInitiating graceful shutdown...");
    });

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Cleanup
    tokio::time::sleep(Duration::from_millis(500)).await;

    let mut desc_file = core.desc_file.lock().unwrap();
    desc_file.flush()?;

    let mut data_file = core.data_file.lock().unwrap();
    data_file.flush()?;

    println!("All data flushed to disk");
    println!("Resources released");
    Ok(())
}

async fn shutdown_signal() {
    signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

fn verify_token(token: &str) -> bool {
    token == AUTH_TOKEN
}

async fn serve_html() -> Html<&'static str> {
    Html(include_str!("../index.html"))
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

async fn handle_get(
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

async fn handle_set(
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

async fn handle_delete(
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

async fn handle_list(
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

async fn handle_list_type(
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

async fn handle_exec(
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

async fn handle_exec_now(
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
