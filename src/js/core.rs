use deno_core::error::AnyError;
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
use crate::kv::core::Core;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub id: u32,
    pub path: String,
    pub payload: serde_json::Value,
    pub headers: Option<HashMap<String, String>>,
    pub event_type: String,
}
pub type Events = Arc<Mutex<VecDeque<Event>>>;
pub type Dones = Arc<Mutex<HashMap<u32, oneshot::Sender<serde_json::Value>>>>;
pub struct Runt {
    pub events: Events,
    dones: Dones,
    kv: Arc<Core>,
}
impl Runt {
    pub fn new(kv: Arc<kv::core::Core>) -> Runt {
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let dones = Arc::new(Mutex::new(HashMap::new()));

        let events_clone = Arc::clone(&events);
        let dones_clone = Arc::clone(&dones);

        run(Arc::clone(&kv), Arc::clone(&events), Arc::clone(&dones));
        Runt { events, dones, kv }
    }

    pub fn add_event_to_queue(&self, event: Event) {
        let mut queue = self.events.lock().unwrap();
        queue.push_back(event);
    }
}

async fn run_js(
    file_path: &str,
    kv: Arc<kv::core::Core>,
    events: Events,
    dones: Dones,
) -> Result<(), AnyError> {
    let main_module = deno_core::resolve_path(file_path, &std::env::current_dir()?)?;
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        extensions: vec![runjs::init_ops_and_esm()],
        ..Default::default()
    });
    {
        let op_state = js_runtime.op_state();
        let mut op_state = op_state.borrow_mut();
        op_state.put::<Arc<kv::core::Core>>(kv);
        op_state.put::<Events>(events);
        op_state.put::<Dones>(dones);
    }

    let mod_id = js_runtime.load_main_es_module(&main_module).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(Default::default()).await?;
    result.await
}

fn run(core: Arc<kv::core::Core>, events: Events, dones: Dones) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let path = format!("./humpback-data/scripts/{}", "example.js".to_string());
    if let Err(error) = runtime.block_on(run_js(&path, core, events, dones)) {
        eprintln!("error: {}", error);
    }
}
