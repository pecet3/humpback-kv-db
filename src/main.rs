mod http_server;
mod internal;
mod js;
mod kv;
mod sql;
mod state;
const DIR_PATH: &str = "./humpback-data";
const USER_STORE_PATH: &str = "./humpback-data/store.sql";
const INTERNAL_STORE_PATH: &str = "./humpback-data/internal.sql";

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

    match http_server::router::run() {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
        }
    }
}
