use deno_core::error::AnyError;
use deno_core::extension;
use deno_core::v8;
use std::rc::Rc;
use std::sync::Arc;

use crate::database;
use crate::js::op_db;
use crate::js::op_file;
use crate::js::op_http;
use database::core::Core;
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
    op_http::op_http_get,
  ],
 esm_entry_point = "ext:runjs/runtime.js",
 esm = [dir "src/js", "runtime.js"],
);

pub struct Runtime {
    runtime: deno_core::JsRuntime,
}
impl Runtime {
    pub fn new(core: Arc<database::core::Core>) -> Self {
        let mut js_runtime: deno_core::JsRuntime =
            deno_core::JsRuntime::new(deno_core::RuntimeOptions {
                module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
                extensions: vec![runjs::init_ops_and_esm()],
                ..Default::default()
            });
        {
            let op_state = js_runtime.op_state();
            let mut op_state = op_state.borrow_mut();
            op_state.put::<Arc<database::core::Core>>(core);
        }
        Runtime {
            runtime: js_runtime,
        }
    }
    pub fn execute(&mut self, script: &str) -> Result<v8::Global<v8::Value>, AnyError> {
        let script = script.to_string();
        return self.runtime.execute_script("", script);
    }
}

pub fn spawn_js_runtime(core: Arc<Core>) -> Arc<Sender<String>> {
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    thread::spawn(move || {
        let mut runtime = Runtime::new(core);
        for script in rx {
            match runtime.execute(&script) {
                Ok(_) => println!("[JS OK] Script executed"),
                Err(e) => eprintln!("[JS ERROR] {:?}", e),
            }
        }
    });

    return Arc::new(tx);
}
