use core::fmt;
use std::io::{Read, Seek, SeekFrom, Write};
use std::{cell::RefMut, collections::HashMap, fs::File, process, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{io_service, store};
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]

pub struct ObjectDescriptor {
    pub key: String,
    pub kind: Kind,
    pub offset: u64,
    pub size: u64,
}

pub struct Object {
    pub desc: ObjectDescriptor,
    pub data: Vec<u8>,
}
pub struct ObjectService {
    pub objects_map: HashMap<String, Object>,
}

impl ObjectService {
    pub fn new() -> ObjectService {
        ObjectService {
            objects_map: HashMap::new(),
        }
    }
    pub fn load_objects_desc(&mut self, mut file: RefMut<File>) {
        const RECORD_SIZE: usize = 255 + 8 + 8 + 1;
        let mut buffer = vec![0u8; RECORD_SIZE];

        while let Ok(_) = file.read_exact(&mut buffer) {
            let raw_key = &buffer[..255];
            let key = match std::str::from_utf8(raw_key) {
                Ok(key) => key.trim_end_matches(char::from(0)).to_string(),
                Err(_) => continue,
            };
            let kind = Kind::from_u8(buffer[255]);

            let offset = u64::from_le_bytes(buffer[256..264].try_into().unwrap());
            let size = u64::from_le_bytes(buffer[264..272].try_into().unwrap());

            let key_copy = key.clone();
            let object_descriptor = ObjectDescriptor {
                key,
                kind,
                offset,
                size,
            };
            let object = Object {
                desc: object_descriptor,
                data: vec![],
            };
            self.objects_map.insert(key_copy, object);
        }
    }

    pub fn load_objects_data(&mut self, mut file: RefMut<File>) {
        for object in self.objects_map.values_mut() {
            let data =
                io_service::read_object_from_file(&mut file, object.desc.offset, object.desc.size);

            object.data = data.unwrap();
        }
    }
}
