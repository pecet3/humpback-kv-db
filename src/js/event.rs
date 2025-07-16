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

use std::sync::atomic::{AtomicI32, Ordering};

static CURRENT_EVENT_ID: AtomicI32 = AtomicI32::new(1);

fn next_id() -> i32 {
    CURRENT_EVENT_ID.fetch_add(1, Ordering::Relaxed)
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
