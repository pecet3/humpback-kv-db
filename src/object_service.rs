use core::fmt;
use std::{collections::HashMap, process, str::FromStr, sync::Arc};

use rusqlite::ffi::Error;

use crate::store;
#[derive(Debug, Clone)]
pub enum Kind {
    Number,
    Boolean,
    String,
    Json,
    Struct,
}
impl FromStr for Kind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "number" => Ok(Kind::Number),
            "boolean" => Ok(Kind::Boolean),
            "string" => Ok(Kind::String),
            "json" => Ok(Kind::Json),
            "struct" => Ok(Kind::Struct),
            _ => Err(()),
        }
    }
}
impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Kind::Number => "number",
            Kind::Boolean => "boolean",
            Kind::String => "string",
            Kind::Json => "json",
            Kind::Struct => "struct",
        };
        write!(f, "{}", s)
    }
}
impl Kind {
    pub fn from_u8(value: u8) -> Kind {
        match value {
            0 => Kind::Number,
            1 => Kind::Boolean,
            2 => Kind::String,
            3 => Kind::Json,
            4 => Kind::Struct,
            _ => Kind::Struct,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Kind::Number => 0,
            Kind::Boolean => 1,
            Kind::String => 2,
            Kind::Json => 3,
            Kind::Struct => 4,
        }
    }
}

pub struct Object {
    pub kind: Kind,
    pub offset: usize,
    pub size: usize,
}
pub struct ObjectService {
    store: Arc<store::StoreDb>,
    pub objects_map: HashMap<String, Object>,
}

impl ObjectService {
    pub fn new(store: Arc<store::StoreDb>) -> ObjectService {
        ObjectService {
            store,
            objects_map: HashMap::new(),
        }
    }
    pub fn load_objects(&mut self) {
        match self.store.read_all() {
            Ok(objects) => {
                for obj_desc in objects.iter() {
                    let key = obj_desc.key.clone();

                    let object = Object {
                        kind: obj_desc.kind.clone(),
                        offset: obj_desc.offset,
                        size: obj_desc.size,
                    };

                    self.objects_map.insert(key.to_string(), object);
                }
                println!("Loaded objects")
            }
            Err(err) => {
                eprintln!("ERROR Loading object descriptions to RAM {:?}", err);
                process::exit(1);
            }
        }
    }
}
