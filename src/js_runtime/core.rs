use deno_core::OpState;
use deno_core::error::AnyError;
use deno_core::extension;
use deno_core::op2;
use std::rc::Rc;
use std::sync::Arc;

use crate::database;
use crate::database::objects::Kind;

extension!(
  runjs,
  ops = [
    op_read_file,
    op_write_file,
    op_remove_file,
    op_db_get_value,
    op_db_set_string,
    op_http_get,
  ],
 esm_entry_point = "ext:runjs/runtime.js",
 esm = [dir "src/js_runtime", "runtime.js"],
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

pub fn run(core: Arc<database::core::Core>) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    if let Err(error) = runtime.block_on(run_js("./example.js", core)) {
        eprintln!("error: {}", error);
    }
}

#[op2(async)]
#[string]
async fn op_read_file(#[string] path: String) -> Result<String, AnyError> {
    let contents = tokio::fs::read_to_string(path).await?;
    Ok(contents)
}

#[op2(async)]
async fn op_write_file(#[string] path: String, #[string] contents: String) -> Result<(), AnyError> {
    tokio::fs::write(path, contents).await?;
    Ok(())
}

#[op2(fast)]
fn op_remove_file(#[string] path: String) -> Result<(), AnyError> {
    std::fs::remove_file(path)?;
    Ok(())
}

// database
#[op2]
#[string]
fn op_db_get_value(state: &mut OpState, #[string] key: String) -> Result<Option<String>, AnyError> {
    let core = state.borrow::<Arc<database::core::Core>>().clone();
    let value = core.get(&key);

    Ok(match value {
        Some(bytes) => Some(String::from_utf8(bytes)?),
        None => None,
    })
}

#[op2(fast)]
fn op_db_set_string(
    state: &mut OpState,
    #[string] key: String,
    #[string] data: String,
) -> Result<(), AnyError> {
    let core = state.borrow::<Arc<database::core::Core>>().clone();

    let data_bytes = data.into_bytes();
    core.set(&key, Kind::String, data_bytes);
    Ok(())
}

// http

#[op2(async)]
#[string]
async fn op_http_get(#[string] url: String) -> Result<String, AnyError> {
    let body = reqwest::get(url).await?.text().await?;
    Ok(body)
}
