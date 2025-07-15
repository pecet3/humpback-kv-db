use deno_core::JsRuntime;
use deno_core::RuntimeOptions;
use deno_core::error::AnyError;
use deno_core::extension;
use deno_core::v8;
use std::rc::Rc;
use std::sync::Arc;

use crate::js::op_db;
use crate::js::op_file;
use crate::js::op_http;
use crate::kv;
use kv::core::Core;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

extension!(
  runjs,
  ops = [
    op_file::file_read,
    op_file::file_write,
    op_file::file_remove,
    op_db::db_get_value,
    op_db::db_set_string,
    op_db::db_set_number,
    op_http::op_http_get,
  ],
 esm_entry_point = "ext:runjs/runtime.js",
 esm = [dir "src/js", "runtime.js"],
);

pub struct Runtime {
    tx_execute: Sender<String>,
}
impl Runtime {
    pub fn new(core: Arc<kv::core::Core>) -> Arc<Self> {
        Arc::new(Runtime {
            tx_execute: spawn_js_runtime(core),
        })
    }
    pub fn execute(&self, script: &str) {
        self.tx_execute.send(script.to_string());
    }
}

fn spawn_js_runtime(core: Arc<Core>) -> Sender<String> {
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let mut js_runtime = JsRuntime::new(RuntimeOptions {
                extensions: vec![runjs::init_ops_and_esm()],
                ..Default::default()
            });

            {
                let op_state = js_runtime.op_state();
                let mut op_state = op_state.borrow_mut();
                op_state.put::<Arc<Core>>(core.clone());
            }
            while let Ok(script) = rx.recv() {
                match js_runtime.execute_script("<anon>", script.clone()) {
                    Ok(_) => {
                        if let Err(e) = js_runtime.run_event_loop(Default::default()).await {
                            eprintln!("[JS EVENT LOOP ERROR] {:?}", e);
                        } else {
                            println!("[JS OK] Script executed");
                        }
                    }
                    Err(e) => {
                        eprintln!("[JS ERROR] {:?}", e);
                    }
                }
            }
        });
    });

    tx
}
