use std::{
    cell::RefCell,
    fs::{self, File, OpenOptions},
    path::Path,
    ptr::null,
    sync::Arc,
};

use clap::error;

use crate::{
    DIR_PATH, io_service as io,
    object_service::{self, Kind, Object, ObjectDescriptor},
    store::{self},
};

pub struct Core {
    objects: object_service::ObjectService,
    pub data_file: RefCell<File>,
    pub desc_file: RefCell<File>,
}
impl Core {
    pub fn new() -> Result<Core, std::io::Error> {
        fs::create_dir_all(DIR_PATH).expect("Unable to create directory with data...");
        let data_file_path = io::get_data_filename("main");
        if !Path::new(&data_file_path).exists() {
            File::create(&data_file_path)?;
        }

        let data_file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open(&data_file_path)?;

        let desc_file_path = io::get_desc_filename("main");
        if !Path::new(&desc_file_path).exists() {
            File::create(&desc_file_path)?;
        }

        let desc_file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open(&desc_file_path)?;
        let mut core = Core {
            objects: object_service::ObjectService::new(),
            data_file: RefCell::new(data_file),
            desc_file: RefCell::new(desc_file),
        };
        core.objects.load_objects_desc(core.desc_file.borrow_mut());
        core.objects.load_objects_data(core.data_file.borrow_mut());
        Ok(core)
    }
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        println!("{:?}", self.objects.objects_map.len());
        match self.objects.objects_map.get(key) {
            Some(value) => {
                let data = value.data.clone();
                Some(data)
            }
            None => None,
        }
    }
    pub fn set(&mut self, key: &str, data: Vec<u8>) {
        let size = data.len();

        let mut file_ref = self.data_file.borrow_mut();
        let offset = io::save_object_in_file(&data, file_ref).unwrap() as usize;
        let mut desc_file_ref = self.desc_file.borrow_mut();
        let desc = ObjectDescriptor {
            key: key.to_string(),
            kind: Kind::String,
            offset: offset as u64,
            size: size as u64,
        };
        let desc_data = bincode::serialize(&desc).unwrap();
        io::save_desc_in_file(&desc_data, desc_file_ref);

        let obj = Object { desc: desc, data };
        self.objects.objects_map.insert(key.to_string(), obj);
    }
}
// use std::fs;

// use crate::{
//     DIR_PATH,
//     groups::{self, ObjectStore},
//     io_service,
//     objects::{Kind, Object_descriptor},
// };

// pub struct Core {
//     pub store: groups::ObjectStore,
//     io: io_service::IoService,
// }
// impl Core {
//     pub fn new() -> Core {
//         let mut core = Core {
//             store: groups::ObjectStore::new(),
//             io: io_service::IoService::new(),
//         };
//         fs::create_dir_all(DIR_PATH).expect("Nie udało się utworzyć katalogu danych");
//         core.store.init().unwrap();
//         return core;
//     }
//     pub fn get_object_by_key_and_group(
//         &self,
//         group_name: &str,
//         key: &str,
//     ) -> Result<Option<&Object_descriptor>, Box<dyn std::error::Error>> {
//         let group = self.store.get_group(group_name).unwrap();

//         let object = group.get_object_descriptor_by_key(key);

//         object
//     }
//     pub fn get_object_by_key(&self, key: &str) -> Option<&Object_descriptor> {
//         for group in self.store.groups.iter() {
//             if let Some(obj) = group.1.table_map.get(key) {
//                 return Some(obj);
//             }
//         }
//         None
//     }
//     pub fn add(&mut self, key: &str, data: Vec<u8>) {
//         let default_group = self.store.groups.get_mut("default").unwrap();
//         let size = data.len();
//         let mut obj = Object_descriptor {
//             data,
//             created_at: 1_656_000_000_000,
//             updated_at: 1_656_100_000_000,
//             last_opened_at: 1_656_200_000_000,
//             next_offset: 1024,
//             kind: Kind::String,
//             offset: 100,
//             size,
//             key: key.to_string(),
//             is_mem_storage: false,
//             header: [0u8; 16],
//             columns: vec![0, 1, 2],
//             free_bytes: 10,
//         };
//         let obj_cpy = obj.clone();
//         let file_ref = default_group.data_file.borrow_mut();
//         let offset = self.io.save_object_on_disk(&obj_cpy, file_ref).unwrap();
//         obj.offset = offset;
//         default_group.table_map.insert(key.to_string(), obj);
//     }
// }
