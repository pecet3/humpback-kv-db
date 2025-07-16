use deno_core::op2;
use deno_core::{OpState, error::AnyError, futures::FutureExt};
use std::{cell::RefCell, rc::Rc};

use crate::js::runtime::{Event, Events};

#[op2]
#[serde]
pub fn op_event_next(state: &mut OpState) -> Result<Option<Event>, AnyError> {
    let queue = state.borrow::<Events>();
    let mut queue = queue.lock().unwrap();
    Ok(queue.pop_front())
}
