use deno_core::error::AnyError;
use deno_core::extension;

use std::rc::Rc;
use std::sync::Arc;

use crate::database;
use crate::js::op_db;
use crate::js::op_file;
use crate::js::op_http;

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

async fn run_js(file_path: &str, core: Arc<database::core::Core>) -> Result<(), AnyError> {
    let main_module = deno_core::resolve_path(file_path, &std::env::current_dir()?)?;
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        extensions: vec![runjs::init_ops_and_esm()],
        ..Default::default()
    });
    {
        let op_state = js_runtime.op_state();
        let mut op_state = op_state.borrow_mut();
        op_state.put::<Arc<database::core::Core>>(core);
    }

    let mod_id = js_runtime.load_main_es_module(&main_module).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(Default::default()).await?;
    result.await
}

pub fn execute(core: Arc<database::core::Core>, script_name: &str) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let path = format!("./humpback-data/scripts/{}", script_name.to_string());
    if let Err(error) = runtime.block_on(run_js(&path, core)) {
        eprintln!("error: {}", error);
    }
}
