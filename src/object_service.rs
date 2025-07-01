use core::fmt;
use std::io::{Read, Seek, SeekFrom, Write};
use std::{cell::RefMut, collections::HashMap, fs::File, process, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::io_service;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key255 {
    pub bytes: Vec<u8>,
}

impl Key255 {
    pub fn new(data: &str) -> Self {
        let bytes = data.as_bytes();
        let mut vec = Vec::with_capacity(255);

        if bytes.len() >= 255 {
            vec.extend_from_slice(&bytes[..255]);
        } else {
            vec.extend_from_slice(bytes);
            vec.resize(255, 0);
        }

        Key255 { bytes: vec }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn to_string(&self) -> String {
        let trimmed = self
            .bytes
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect();
        trimmed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDescriptor {
    pub key: Key255,
    pub kind: Kind,
    pub offset: u64,
    pub size: u64,
}
#[derive(Debug)]
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
        const RECORD_SIZE: usize = 283;
        let mut buffer = vec![0u8; RECORD_SIZE];
        let meta = file.metadata();
        println!("{:?}", meta);

        while let Ok(_) = file.read_exact(&mut buffer) {
            match bincode::deserialize::<ObjectDescriptor>(&buffer) {
                Ok(object_descriptor) => {
                    let key_copy = object_descriptor.key.to_string();
                    let object = Object {
                        desc: object_descriptor,
                        data: vec![],
                    };
                    self.objects_map.insert(key_copy, object);
                }
                Err(e) => {
                    println!("Failed to deserialize record: {:?}", e);
                    continue;
                }
            }
        }
        println!("Loaded object descriptions");
    }

    pub fn load_objects_data(&mut self, mut file: RefMut<File>) {
        println!("Loaded object data");

        for object in self.objects_map.values_mut() {
            let data =
                io_service::read_object_from_file(&mut file, object.desc.offset, object.desc.size);

            object.data = data.unwrap();
        }
    }
}
