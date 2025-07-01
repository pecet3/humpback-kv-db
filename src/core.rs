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
    object_service::{self, Kind, Object},
    store::{self},
};

pub struct Core {
    pub store: Arc<store::StoreDb>,
    data: Vec<u8>,
    objects: object_service::ObjectService,
    pub data_file: RefCell<File>,
}
impl Core {
    pub fn new() -> Result<Core, std::io::Error> {
        fs::create_dir_all(DIR_PATH).expect("Unable to create directory with data...");
        let data_file_path = io::get_data_filename("main");
        if !Path::new(&data_file_path).exists() {
            File::create(&data_file_path);
        }

        let data_file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open(&data_file_path)?;

        let store: Arc<store::StoreDb> =
            Arc::new(store::StoreDb::new(format!("{}/{}", DIR_PATH, "store.db")).unwrap());

        let mut core = Core {
            store: store.clone(),
            data: vec![],
            objects: object_service::ObjectService::new(store.clone()),
            data_file: RefCell::new(data_file),
        };
        core.objects.load_objects();
        Ok(core)
    }
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        println!("{:?}", self.objects.objects_map.len());
        match self.objects.objects_map.get(key) {
            Some(value) => {
                let mut file_ref = self.data_file.borrow_mut();

                let data = io::read_object_from_file(file_ref, value.offset, value.size).ok()?;
                Some(data)
            }
            None => None,
        }
    }
    pub fn add(&mut self, key: &str, data: Vec<u8>) {
        let mut obj = Object {
            kind: Kind::String,
            offset: 0,
            size: data.len(),
        };
        let mut file_ref = self.data_file.borrow_mut();
        let offset = io::save_object_in_file(&obj, data, file_ref).unwrap() as usize;
        obj.offset = offset + 255;
        self.objects.objects_map.insert(key.to_string(), obj);

        match self.store.create(&store::ObjectDescriptor {
            created_at: 0,
            updated_at: 0,
            last_opened_at: 0,
            kind: Kind::Boolean,
            offset: offset + 255,
            size: 2,
            key: key.to_string(),
            data: vec![],
        }) {
            Err(e) => {
                println!("Error during saving a file:{:?}", e.);
            }
            _ => {}
        }
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
