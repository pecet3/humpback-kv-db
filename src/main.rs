use std::sync::Arc;
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
    let scr = r#"console.log("Hello", "runjs!");
console.error("Boom!");

const result = db.get("test");
console.log(result);"#;

    let runtime = js::runtime::Runtime::new(Arc::clone(&core));
    runtime.execute(scr);
    match tcp_service::run(Arc::clone(&core)) {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
        }
    }
}
