use deno_core::{OpState, error::AnyError, futures::FutureExt};
use deno_core::{op2, serde_json};
use std::{cell::RefCell, rc::Rc};

use crate::js::event::Event;
use crate::js::runtime::{Events, Results};

#[op2]
#[serde]
pub fn op_event_next(state: &mut OpState) -> Result<Option<Event>, AnyError> {
    let queue = state.borrow::<Events>();
    let mut queue = queue.lock().unwrap();
    Ok(queue.pop_front())
}

#[op2]
#[serde]
pub fn op_event_return(state: &mut OpState, id: i32, #[serde] event_result: serde_json::Value) {
    let results = state.borrow::<Results>();
    let mut results_mut = results.lock().unwrap();
    println!("{:?}", event_result);
    if let Some(sender) = results_mut.remove(&id) {
        let result = sender.send(event_result);
        if result.is_err() {
            println!("ERROR");
        }
    }
}
