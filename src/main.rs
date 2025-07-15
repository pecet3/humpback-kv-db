use std::sync::Arc;

use std::sync::Mutex;
mod database;
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
    let runtime: Arc<Mutex<js::runtime::Runtime>> =
        Arc::new(Mutex::new(js::runtime::Runtime::new(Arc::clone(&core))));

    // Skrypt
    // let scr = r#"
    //     console.log("Hello, runjs!");
    // "#;

    // // Dostęp mutowalny do runtime
    // {
    //     let mut rt = runtime.lock().unwrap();
    //     rt.execute(scr).expect("Script execution failed");
    // }
    match tcp_service::run(Arc::clone(&core)) {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
        }
    }
}
