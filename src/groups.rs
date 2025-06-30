use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    path::Path,
};

use crate::{
    DIR_PATH, groups, io_service,
    objects::{Group, Object_descriptor},
    parser,
};

pub struct ObjectStore {
    pub groups: HashMap<String, Group>, // Now holds Group instances
    pub length: usize,
}

impl ObjectStore {
    pub fn new() -> ObjectStore {
        ObjectStore {
            groups: HashMap::new(),
            length: 0,
        }
    }
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let groups_list_file = io_service::get_groups_list();
        println!("{:?}", groups_list_file);

        if !Path::new(&groups_list_file).exists() {
            panic!("");
        }
        let groups = parser::parse_lines_from_file(&groups_list_file)?;
        println!("{:?}", groups);

        for group in groups {
            let mut new_group = Group::new(&group);
            new_group.init()?;
            self.groups.insert(group.clone(), new_group);
        }

        Ok(())
    }
    /// Adds a new group to the `Groups` collection.
    /// It creates the necessary files for the group if they don't exist.
    pub fn add_group(&mut self, group_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.groups.contains_key(group_name) {
            return Err(format!("Group '{}' already exists.", group_name).into());
        }

        let mut new_group = Group::new(&group_name.to_string());
        new_group.init()?;

        self.groups.insert(group_name.to_string(), new_group);
        self.length = self.groups.len(); // Update the length

        Ok(())
    }

    /// Retrieves a mutable reference to a group by its name.
    pub fn get_group_mut(
        &mut self,
        group_name: &str,
    ) -> Result<&mut Group, Box<dyn std::error::Error>> {
        self.groups
            .get_mut(group_name)
            .ok_or_else(|| format!("Group '{}' not found.", group_name).into())
    }

    /// Retrieves an immutable reference to a group by its name.
    pub fn get_group(&self, group_name: &str) -> Result<&Group, Box<dyn std::error::Error>> {
        self.groups
            .get(group_name)
            .ok_or_else(|| format!("Group '{}' not found.", group_name).into())
    }

    pub fn get_all_groups(&self) -> Vec<&Group> {
        self.groups.values().collect()
    }
}
