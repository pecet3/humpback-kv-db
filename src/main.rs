use std::sync::Arc;

mod http_service;
mod js;
mod kv;

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

    let kv = kv::core::Core::new().expect("Init error");

    match http_service::run(Arc::clone(&kv)) {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
        }
    }
}
