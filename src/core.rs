use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    path::Path,
    ptr::null,
    sync::{Arc, Mutex},
};

use crate::{
    DIR_PATH, io_service as io,
    object_service::{self, Key255, Kind, Object, ObjectDescriptor, ObjectListElement},
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
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        return self.objects.get(key);
    }
    pub async fn set(&self, key: &str, kind: Kind, data: Vec<u8>) {
        let size = data.len();

        let data_file = Arc::clone(&self.data_file);
        let data_clone = data.clone();

        let offset =
            tokio::task::spawn_blocking(move || io::save_object_in_file(&data_clone, data_file))
                .await
                .expect("spawn_blocking failed")
                .expect("Failed to write data") as usize;

        let desc = ObjectDescriptor {
            key: Key255::new(key),
            kind: kind.clone(),
            offset: offset as u64,
            size: size as u64,
        };

        let desc_data = bincode::serialize(&desc).unwrap();
        let desc_file = Arc::clone(&self.desc_file);

        tokio::task::spawn_blocking(move || io::save_desc_in_file(&desc_data, desc_file))
            .await
            .expect("spawn_blocking failed")
            .expect("Failed to write descriptor");

        let obj = Object {
            desc: ObjectDescriptor {
                key: Key255::new(key),
                kind: kind,
                offset: offset as u64,
                size: size as u64,
            },
            data,
        };
        self.objects.set(obj).unwrap();
    }
    pub async fn list(&self) -> Result<Vec<ObjectListElement>, Box<dyn Error + Send + Sync>> {
        return self.objects.list();
    }
}
