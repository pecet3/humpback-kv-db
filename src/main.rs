use std::sync::Arc;

use std::sync::Mutex;
mod database;
mod http_service;
mod js;
mod tcp_service;
const DIR_PATH: &str = "./humpback-data";

fn main() {
    println!(
        r#"
        ────────────────────────────────────────────
          🐋 Humpback KV Database
          Licensed under MIT/Apache-2.0
          
          Created by Jakub Pacewicz 
          http://github.com/pecet3/humpback-kv-db
        ────────────────────────────────────────────
        "#
    );
    let core = database::core::Core::new().expect("Init error");

    match http_service::run(Arc::clone(&core)) {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
        }
    }
}
