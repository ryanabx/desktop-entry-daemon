use std::{
    collections::{HashMap, HashSet},
    fs::remove_file,
    path::PathBuf,
    process::Command,
};

use zbus::names::OwnedUniqueName;

#[derive(Clone, Debug)]
pub struct EntryCatalog {
    pub owned_resources: HashMap<u32, (Vec<DesktopEntry>, Vec<IconEntry>)>,
    pub change_handlers: HashSet<u32>,
}

impl EntryCatalog {
    pub fn new() -> Self {
        Self {
            owned_resources: HashMap::new(),
            change_handlers: HashSet::new(),
        }
    }

    pub fn add_desktop_entry(&mut self, name: u32, entry: DesktopEntry) {
        if !self.owned_resources.contains_key(&name) {
            self.owned_resources
                .insert(name.clone(), (Vec::new(), Vec::new()));
        }
        self.owned_resources.get_mut(&name).unwrap().0.push(entry);
        if self.change_handlers.is_empty() {
            let _ = Command::new("update-desktop-database").spawn();
        }
    }

    pub fn add_icon(&mut self, name: u32, entry: IconEntry) {
        if !self.owned_resources.contains_key(&name) {
            self.owned_resources
                .insert(name.clone(), (Vec::new(), Vec::new()));
        }
        self.owned_resources.get_mut(&name).unwrap().1.push(entry);
        if self.change_handlers.is_empty() {
            let _ = Command::new("update-desktop-database").spawn();
        }
    }

    pub fn remove_owner(&mut self, name: u32) {
        if !self.owned_resources.contains_key(&name) {
            return;
        }
        let (entries, icons) = self.owned_resources.get(&name).unwrap();
        for entry in entries {
            let _ = entry.clone().delete_self();
        }
        for icon in icons {
            let _ = icon.clone().delete_self();
        }

        self.owned_resources.remove(&name);
        if self.change_handlers.is_empty() {
            let _ = Command::new("update-desktop-database").spawn();
        }
        log::info!("Removed owner with name {:?}", name);
    }
}

#[derive(Clone, Debug)]
pub struct DesktopEntry {
    pub appid: String,
    pub path: PathBuf,
}

impl DesktopEntry {
    fn delete_self(self) -> Result<(), std::io::Error> {
        remove_file(&self.path)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct IconEntry {
    pub icon_name: String,
    pub icon_path: PathBuf,
}

impl IconEntry {
    fn delete_self(self) -> Result<(), std::io::Error> {
        remove_file(&self.icon_path)?;
        Ok(())
    }
}
