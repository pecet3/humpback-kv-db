use std::{
    fs::{self, File, OpenOptions},
    path::Path,
    ptr::null,
    sync::{Arc, Mutex},
};

use clap::error;

use crate::{
    DIR_PATH, io_service as io,
    object_service::{self, Key255, Kind, Object, ObjectDescriptor},
};

pub struct Core {
    objects: object_service::ObjectService,
    pub data_file: Arc<Mutex<File>>,
    pub desc_file: Arc<Mutex<File>>,
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
            data_file: Arc::new(Mutex::new(data_file)),
            desc_file: Arc::new(Mutex::new(desc_file)),
        };
        core.objects.load_objects_desc(Arc::clone(&core.desc_file));
        core.objects.load_objects_data(Arc::clone(&core.data_file));
        Ok(core)
    }
    pub async fn get(mut self, key: &str) -> Option<Vec<u8>> {
        return self.objects.get(key).await;
    }
    pub async fn set(&mut self, key: &str, kind: Kind, data: Vec<u8>) {
        let size = data.len();

        let offset = io::save_object_in_file(&data, Arc::clone(&self.data_file)).unwrap() as usize;
        let desc = ObjectDescriptor {
            key: Key255::new(key),
            kind: kind.clone(),
            offset: offset as u64,
            size: size as u64,
        };

        let desc_data = bincode::serialize(&desc).unwrap();
        println!("data: {:?}", data);
        match io::save_desc_in_file(&desc_data, Arc::clone(&self.desc_file)) {
            Err(e) => panic!("{}", e),

            _ => {
                println!("set an object")
            }
        }
        let obj = Object {
            desc: ObjectDescriptor {
                key: Key255::new(key),
                kind: kind,
                offset: offset as u64,
                size: size as u64,
            },
            data,
        };
        self.objects.set(obj).await.unwrap();
    }
}
