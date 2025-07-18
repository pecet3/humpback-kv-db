use crate::kv;
use crate::kv::objects::Kind;
use deno_core::OpState;
use deno_core::error::AnyError;
use deno_core::op2;
use deno_core::serde_json::{self, json};
use std::sync::Arc;

#[op2]
#[serde]
pub fn op_kv_get_value(
    state: &mut OpState,
    #[string] key: String,
) -> Result<serde_json::Value, AnyError> {
    let core = state.borrow::<Arc<kv::core::Core>>().clone();
    let value = core.get(&key);

    Ok(match value {
        Some(object) => match object.desc.kind {
            Kind::String => {
                let s = String::from_utf8(object.data)?;
                json!(s)
            }
            Kind::Number => {
                let bytes: [u8; 8] = object.data.as_slice().try_into()?;
                let n = f64::from_le_bytes(bytes);
                json!(n)
            }
            _ => {
                let s = String::from_utf8_lossy(&object.data).to_string();
                json!(s)
            }
        },
        None => json!(null),
    })
}
#[op2]
#[string]
pub fn op_kv_get_kind(state: &mut OpState, #[string] key: String) -> Result<String, AnyError> {
    let core = state.borrow::<Arc<kv::core::Core>>().clone();
    let value = core.get(&key);

    Ok(match value {
        Some(object) => Kind::to_string(&object.desc.kind),
        None => "null".to_string(),
    })
}
// #[op2]
// #[serde]
// pub fn op_kv_get_value(
//     state: &mut OpState,
//     #[string] key: String,
// ) -> Result<Option<AnyValue>, AnyError> {
//     let core = state.borrow::<Arc<kv::core::Core>>().clone();
//     let value = core.get(&key);

//     Ok(match value {
//         Some(object) => match object.desc.kind {
//             Kind::String => {
//                 let string = String::from_utf8(object.data)?;
//                 Some(AnyValue::String(string))
//             }
//             Kind::Number => {
//                 let bytes: [u8; 8] = object.data.as_slice().try_into()?;
//                 let f64 = f64::from_le_bytes(bytes);
//                 Some(AnyValue::Number(f64))
//             }
//             _ => {
//                 let string = String::from_utf8(object.data)?;
//                 Some(AnyValue::String(string))
//             }
//         },
//         None => None,
//     })
// }

#[op2(fast)]
pub fn op_kv_set_string(
    state: &mut OpState,
    #[string] key: String,
    #[string] data: String,
) -> Result<(), AnyError> {
    let core = state.borrow::<Arc<kv::core::Core>>().clone();
    let data_bytes = data.into_bytes();
    core.set(&key, Kind::String, data_bytes);
    Ok(())
}

#[op2(fast)]
pub fn op_kv_set_number(
    state: &mut OpState,
    #[string] key: String,
    data: f64,
) -> Result<(), AnyError> {
    let core = state.borrow::<Arc<kv::core::Core>>().clone();
    println!("{}", data);
    let data_bytes = data.to_le_bytes().to_vec();
    core.set(&key, Kind::Number, data_bytes);
    Ok(())
}

#[op2]
#[serde]
pub fn op_kv_set_object(
    state: &mut OpState,
    #[string] key: String,
    #[serde] data: serde_json::Value,
) -> Result<(), AnyError> {
    let core = state.borrow::<Arc<kv::core::Core>>().clone();
    let bytes = serde_json::to_vec(&data).unwrap();
    core.set(&key, Kind::Object, bytes);
    Ok(())
}
