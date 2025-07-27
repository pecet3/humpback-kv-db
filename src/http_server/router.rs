use crate::{
    USER_STORE_PATH,
    http_server::handlers,
    internal::{self, db::InternalDb},
    js::{self, event::Event, runtime::Runtime},
    kv::{self, core::Core, objects::Kind},
    sql::{self, db::Db},
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

pub const AUTH_TOKEN: &str = "humpback_secret_token_2024";

#[derive(Clone)]
pub struct AppState {
    pub core: Arc<Core>,
    pub runtime: Arc<Runtime>,
    pub db: Arc<Db>,
    pub internal_db: Arc<InternalDb>,
}

#[tokio::main]
pub async fn run() -> Result<(), Box<dyn Error>> {
    let core = kv::core::Core::new().unwrap();
    let runtime: Arc<Runtime> = js::runtime::Runtime::new(Arc::clone(&core));
    let db = Arc::new(sql::db::Db::new(USER_STORE_PATH).unwrap());
    let internal_db = Arc::new(internal::db::InternalDb::new().unwrap());
    let state = AppState {
        core: Arc::clone(&core),
        runtime,
        db,
        internal_db,
    };

    let app = Router::new()
        .route("/", get(handlers::serve_html))
        .route("/get", post(handlers::handle_get))
        .route("/set", post(handlers::handle_set))
        .route("/delete", post(handlers::handle_delete))
        .route("/list", post(handlers::handle_list))
        .route("/listType", post(handlers::handle_list_type))
        .route("/exec", post(handlers::handle_exec))
        .route("/execNow", post(handlers::handle_exec_now))
        .route("/sql/exec", post(handlers::handle_sql_exec))
        .route("/sql/query", post(handlers::handle_sql_query))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    println!("Humpback KV Database HTTP Server is listening on 127.0.0.1:8080");

    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        println!("Exit signal received\nInitiating graceful shutdown...");
    });

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

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
