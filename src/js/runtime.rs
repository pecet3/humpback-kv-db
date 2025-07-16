use deno_core::extension;
use deno_core::futures::channel::oneshot;
use deno_core::serde_json;

use deno_core::serde_json::json;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

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
use std::sync::atomic::{AtomicI32, Ordering};

static NEXT_ID: AtomicI32 = AtomicI32::new(1);

fn next_id() -> i32 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub id: i32,
    pub path: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub code: String,
}
impl Event {
    pub fn new_code_event(code: String) -> Event {
        Event {
            id: next_id(),
            code,
            event_type: "code".to_string(),
            path: "".to_string(),
            payload: serde_json::json!({}),
        }
    }

    pub fn new_request_event(path: String, payload: serde_json::Value) -> Event {
        Event {
            id: next_id(),
            code: "".to_string(),
            event_type: "request".to_string(),
            path,
            payload,
        }
    }
}
pub type Events = Arc<Mutex<VecDeque<Event>>>;
pub type Dones = Arc<Mutex<HashMap<u32, oneshot::Sender<serde_json::Value>>>>;
pub struct Runtime {
    events: Arc<Mutex<VecDeque<Event>>>,
}
impl Runtime {
    pub fn new(core: Arc<Core>) -> Arc<Self> {
        let events: Arc<Mutex<VecDeque<Event>>> = Arc::new(Mutex::new(VecDeque::new()));
        spawn_js_runtime(Arc::clone(&core), Arc::clone(&events));
        Arc::new(Runtime { events })
    }

    pub fn push_event(&self, event: Event) {
        let mut queue = self.events.lock().unwrap();
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
                "./humpback-data/scripts/eventLoop.js",
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

            let mod_id = js_runtime.load_main_es_module(&main_module).await.unwrap();
            let result = js_runtime.mod_evaluate(mod_id);
            js_runtime.run_event_loop(Default::default()).await.unwrap();

            result.await.unwrap();
        });
    });
}
