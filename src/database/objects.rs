use core::fmt;
use std::error::Error;
use std::io::Read;
use std::sync::{Mutex, RwLock};
use std::{collections::HashMap, fs::File, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::database::io_service;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Kind {
    Number,
    Boolean,
    String,
    Json,
    Blob,
    Object,
    Js,
}

impl FromStr for Kind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "number" => Ok(Kind::Number),
            "boolean" => Ok(Kind::Boolean),
            "string" => Ok(Kind::String),
            "json" => Ok(Kind::Json),
            "blob" => Ok(Kind::Blob), // Changed "Blob" to "blob" for consistency with to_lowercase()
            "object" => Ok(Kind::Object),
            "js" => Ok(Kind::Js),
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
            Kind::Blob => "Blob",
            Kind::Object => "Object",
            Kind::Js => "Js",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key256 {
    pub bytes: Vec<u8>,
}

impl Key256 {
    pub fn new(data: &str) -> Self {
        let bytes = data.as_bytes();
        let mut vec = Vec::with_capacity(256);

        if bytes.len() >= 256 {
            vec.extend_from_slice(&bytes[..256]);
        } else {
            vec.extend_from_slice(bytes);
            vec.resize(256, 0);
        }

        Key256 { bytes: vec }
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

#[derive(Debug, Clone)]
pub struct ObjectListElement {
    pub key: String,
    pub size: u64,
    pub kind: Kind,
    pub offset: u64,
}
const RECORD_SIZE: usize = 293;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDescriptor {
    pub key: Key256,
    pub kind: Kind,
    pub offset: u64,
    pub size: u64,
    pub is_deleted: bool,
    pub desc_offset: u64,
}
#[derive(Debug, Clone)]
pub struct Object {
    pub desc: ObjectDescriptor,
    pub data: Vec<u8>,
}

pub struct ObjectService {
    pub objects_map: RwLock<HashMap<String, Object>>,
}

impl ObjectService {
    pub fn new() -> ObjectService {
        ObjectService {
            objects_map: RwLock::new(HashMap::new()),
        }
    }

    pub fn load_objects_desc(&self, file: Arc<Mutex<File>>) {
        let mut buffer = vec![0u8; RECORD_SIZE];
        let mut file = file.lock().expect("Failed to lock desc_file");

        let mut objects_by_key: HashMap<String, Vec<Object>> = HashMap::new();
        while let Ok(_) = file.read_exact(&mut buffer) {
            match bincode::deserialize::<ObjectDescriptor>(&buffer) {
                Ok(object_descriptor) => {
                    let key_copy = object_descriptor.key.to_string();
                    let object = Object {
                        desc: object_descriptor.clone(),
                        data: vec![],
                    };

                    objects_by_key.entry(key_copy).or_default().push(object);
                }
                Err(e) => {
                    println!("Failed to deserialize record: {:?}", e);
                    continue;
                }
            }
        }

        let filtered_objects: Vec<(String, Object)> = objects_by_key
            .into_iter()
            .filter_map(|(key, objects)| {
                if objects.iter().any(|obj| obj.desc.is_deleted) {
                    None
                } else {
                    Some((key, objects[0].clone()))
                }
            })
            .collect();
        let hash_map_len = filtered_objects.len();

        match self.objects_map.write() {
            Ok(mut map) => {
                for (key, object) in filtered_objects {
                    map.insert(key, object);
                }
            }
            Err(e) => {
                eprintln!("Loading object error: {}", e)
            }
        }
        println!(
            "Loaded object descriptors\nObjects in memory: {}",
            hash_map_len
        );
    }

    pub fn load_objects_data(&mut self, file: Arc<Mutex<File>>) {
        match self.objects_map.get_mut() {
            Ok(map) => {
                for object in map {
                    let data = io_service::read_object_from_file(
                        Arc::clone(&file),
                        object.1.desc.offset,
                        object.1.desc.size,
                    );
                    object.1.data = data.unwrap();
                }
            }
            Err(e) => {
                eprintln!("Loading object error: {:?}", e)
            }
        }
        println!("Loaded object data");
    }
    pub fn get_data(&self, key: &str) -> Option<Vec<u8>> {
        let map = self.objects_map.read();
        match map {
            Ok(map) => match map.get(key) {
                Some(object) => Some(object.data.clone()),
                None => None,
            },
            Err(_) => None,
        }
    }
    pub fn get_desc(&self, key: &str) -> Option<ObjectDescriptor> {
        let map = self.objects_map.read();
        match map {
            Ok(map) => match map.get(key) {
                Some(object) => Some(object.desc.clone()),
                None => None,
            },
            Err(_) => None,
        }
    }
    pub fn set(&self, object: Object) -> Result<(), Box<dyn Error>> {
        match self.objects_map.write() {
            Ok(mut map) => {
                map.insert(object.desc.key.to_string(), object);
                Ok(())
            }
            Err(e) => {
                let msg = format!("Poisoned lock: {}", e);
                Err(msg.into())
            }
        }
    }
    pub fn delete(&self, key: String) -> Result<Object, Box<dyn Error + Send + Sync>> {
        match self.objects_map.write() {
            Ok(mut map) => {
                let obj = map.get(&key).ok_or("Object not found")?;
                let mut obj_copy = obj.clone();
                obj_copy.desc.is_deleted = true;
                map.remove(&key);
                Ok(obj_copy)
            }
            Err(e) => {
                let msg = format!("Poisoned lock: {}", e);
                Err(msg.into())
            }
        }
    }
    pub fn list(&self) -> Result<Vec<ObjectListElement>, Box<dyn Error + Send + Sync>> {
        match self.objects_map.read() {
            Ok(map) => {
                let mut list: Vec<ObjectListElement> = map
                    .iter()
                    .map(|(key, obj)| ObjectListElement {
                        key: key.clone(),
                        kind: obj.desc.kind.clone(),
                        size: obj.desc.size,
                        offset: obj.desc.offset,
                    })
                    .collect();

                list.sort_by_key(|elem| elem.offset);
                Ok(list)
            }
            Err(e) => {
                let msg = format!("Poisoned lock: {}", e);
                Err(msg.into())
            }
        }
    }
    pub fn list_by_kind(
        &self,
        kind: Kind,
    ) -> Result<Vec<ObjectListElement>, Box<dyn Error + Send + Sync>> {
        match self.objects_map.read() {
            Ok(map) => {
                let mut list: Vec<ObjectListElement> = map
                    .iter()
                    .filter(|(_, obj)| obj.desc.kind == kind)
                    .map(|(key, obj)| ObjectListElement {
                        key: key.clone(),
                        kind: obj.desc.kind.clone(),
                        size: obj.desc.size,
                        offset: obj.desc.offset,
                    })
                    .collect();

                list.sort_by_key(|elem| elem.offset);
                Ok(list)
            }
            Err(e) => {
                let msg = format!("Poisoned lock: {}", e);
                Err(msg.into())
            }
        }
    }
}
