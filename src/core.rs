use std::fs;

use crate::{
    DIR_PATH,
    groups::{self, ObjectStore},
    objects::{Kind, Object_descriptor},
};

pub struct Core {
    pub store: groups::ObjectStore,
}
impl Core {
    pub fn new() -> Core {
        let mut core = Core {
            store: groups::ObjectStore::new(),
        };
        fs::create_dir_all(DIR_PATH).expect("Nie udało się utworzyć katalogu danych");
        core.store.init().unwrap();
        return core;
    }
    pub fn get_object_by_key_and_group(
        &self,
        group_name: &str,
        key: &str,
    ) -> Result<Option<&Object_descriptor>, Box<dyn std::error::Error>> {
        let group = self.store.get_group(group_name).unwrap();

        let object = group.get_object_descriptor_by_key(key);

        object
    }
    pub fn get_object_by_key(&self, key: &str) {
        for group in self.store.groups.iter() {
            let result = group.1.table_map.get(key);
        }
    }
    pub fn add(&mut self, key: &str, data: Vec<u8>) {
        let default_group = self.store.groups.get_mut("default").unwrap();
        let size = data.len();
        let obj = Object_descriptor {
            data,
            created_at: 1_656_000_000_000,
            updated_at: 1_656_100_000_000,
            last_opened_at: 1_656_200_000_000,
            next_offset: 1024,
            kind: Kind::String,
            offset: 100,
            size,
            key: key.to_string(),
            is_mem_storage: false,
            header: [0u8; 16],
            columns: vec![0, 1, 2],
            free_bytes: 10,
        };
        let obj_cpy = obj.clone();
        default_group.table_map.insert(key.to_string(), obj);
        _ = default_group.save_on_disk(&obj_cpy);
    }
}
