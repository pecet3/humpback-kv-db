use deno_core::JsRuntime;
use deno_core::RuntimeOptions;
use deno_core::error::AnyError;
use deno_core::extension;
use deno_core::serde_json::json;
use deno_core::v8;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::event;

use crate::js::core::Event;
use crate::js::op_event;
use crate::js::op_file;
use crate::js::op_http;
use crate::js::op_kv;
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
    op_kv::kv_get_value,
    op_kv::kv_set_string,
    op_kv::kv_set_number,
    op_http::op_http_get,
    op_event::op_event_next,
  ],
 esm_entry_point = "ext:runjs/runtime.js",
 esm = [dir "src/js", "runtime.js"],
);

pub struct Runtime {
    events: Arc<Mutex<VecDeque<Event>>>,
}
impl Runtime {
    pub fn new(core: Arc<Core>) -> Arc<Self> {
        let events: Arc<Mutex<VecDeque<Event>>> = Arc::new(Mutex::new(VecDeque::new()));
        spawn_js_runtime(Arc::clone(&core), Arc::clone(&events));
        Arc::new(Runtime { events })
    }

    pub fn execute(&self, event: Event) {
        let mut queue = self.events.lock().unwrap();
        println!("pushing event {:?}", event);
        queue.push_back(event);
    }
}
fn spawn_js_runtime(core: Arc<Core>, events: Arc<Mutex<VecDeque<Event>>>) {
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let main_module = deno_core::resolve_path(
                "./humpback-data/scripts/example.js",
                &std::env::current_dir().unwrap(),
            )
            .unwrap();

            let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
                module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
                extensions: vec![runjs::init_ops_and_esm()],
                ..Default::default()
            });

            {
                let op_state = js_runtime.op_state();
                let mut op_state = op_state.borrow_mut();
                op_state.put::<Arc<Core>>(Arc::clone(&core));
                op_state.put::<Arc<Mutex<VecDeque<Event>>>>(Arc::clone(&events));
            }

            // Wczytaj i uruchom JS moduł
            let mod_id = js_runtime.load_main_es_module(&main_module).await.unwrap();
            let result = js_runtime.mod_evaluate(mod_id);
            js_runtime.run_event_loop(Default::default()).await.unwrap();

            result.await.unwrap();
        });
    });
}
