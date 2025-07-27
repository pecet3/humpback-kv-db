use deno_core::extension;

use deno_core::serde_json;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::USER_STORE_PATH;
use crate::js::event::Event;
use crate::js::op_event;
use crate::js::op_http;
use crate::js::op_kv;
use crate::js::op_sql;
use crate::kv;
use crate::sql;
use kv::core::Core;
use std::thread;
extension!(
  runjs,
  ops = [
    op_kv::op_kv_get_value,
    op_kv::op_kv_get_kind,
    op_kv::op_kv_set_string,
    op_kv::op_kv_set_number,
    op_kv::op_kv_set_object,
    op_sql::op_sql_exec,
    op_sql::op_sql_query,
    op_http::op_http_get,
    op_http::op_http_post,
    op_http::op_http_delete,
    op_http::op_http_put,
    op_event::op_event_next,
    op_event::op_event_return,
  ],
 esm_entry_point = "ext:runjs/runtime.js",
 esm = [dir "src/js", "runtime.js"],
);

enum RuntimeCommand {
    Resume,
    Stop,
    Restart,
    Shutdown,
}

pub type Events = Arc<Mutex<VecDeque<Event>>>;
pub type Results = Arc<Mutex<HashMap<i32, oneshot::Sender<serde_json::Value>>>>;
pub struct Runtime {
    events: Events,
    results: Results,
}
impl Runtime {
    pub fn new(core: Arc<Core>) -> Arc<Self> {
        let events: Events = Arc::new(Mutex::new(VecDeque::new()));
        let results: Results = Arc::new(Mutex::new(HashMap::new()));
        spawn_js_runtime(Arc::clone(&core), Arc::clone(&events), Arc::clone(&results));
        Arc::new(Runtime { events, results })
    }

    pub fn push_event(&self, event: Event) -> oneshot::Receiver<serde_json::Value> {
        let id = event.id.clone();
        let (tx, rx) = oneshot::channel();
        {
            let mut queue = self.events.lock().unwrap();
            queue.push_back(event);
        }
        {
            let mut map = self.results.lock().unwrap();
            map.insert(id, tx);
        }

        rx
    }
}

fn spawn_js_runtime(
    core: Arc<Core>,
    events: Events,
    results: Results,
) -> UnboundedSender<RuntimeCommand> {
    let (tx, mut rx): (
        UnboundedSender<RuntimeCommand>,
        UnboundedReceiver<RuntimeCommand>,
    ) = unbounded_channel();

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime");

        let mut should_restart = true;

        while should_restart {
            should_restart = false;

            let res: Result<(), ()> = rt.block_on(async {
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

                let db = sql::db::Db::new(USER_STORE_PATH).unwrap();
                {
                    let op_state = js_runtime.op_state();
                    let mut op_state = op_state.borrow_mut();
                    op_state.put::<Arc<Core>>(Arc::clone(&core));
                    op_state.put::<Events>(Arc::clone(&events));
                    op_state.put::<Results>(Arc::clone(&results));
                    op_state.put::<sql::db::Db>(db);
                }

                let mod_id = js_runtime.load_main_es_module(&main_module).await.unwrap();
                let eval = js_runtime.mod_evaluate(mod_id);

                tokio::select! {
                    _ = js_runtime.run_event_loop(Default::default()) => {
                        if let Err(e) = eval.await {
                            eprintln!("[JS Runtime] 💥 Evaluation error: {e}");
                            return Err(());
                        }
                    }
                    Some(cmd) = rx.recv() => {
                        match cmd {
                            RuntimeCommand::Stop => {
                                eprintln!("[JS Runtime] ⏸️ Paused");
                                loop {
                                    match rx.recv().await {
                                        Some(RuntimeCommand::Resume) => {
                                            eprintln!("[JS Runtime] ▶️ Resumed");
                                            break;
                                        },
                                        Some(RuntimeCommand::Shutdown) => {
                                            eprintln!("[JS Runtime] 🛑 Shutdown during pause");
                                            return Ok(());
                                        },
                                        _ => continue,
                                    }
                                }
                            }
                            RuntimeCommand::Restart => {
                                eprintln!("[JS Runtime] 🔁 Restart requested");
                                return Err(());
                            }
                            RuntimeCommand::Shutdown => {
                                eprintln!("[JS Runtime] 🛑 Shutdown");
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                }

                Ok(())
            });

            if res.is_err() {
                should_restart = true;
                eprintln!("[JS Runtime] 🔁 Restarting...");
            }
        }
    });

    tx
}
