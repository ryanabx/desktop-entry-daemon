use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
};

use async_std::sync::{Arc, Mutex};

use crate::{daemon::Daemon, types::EntryCatalog};

pub fn get_data_dir(clean: bool) -> PathBuf {
    let home = env::var("HOME").expect("can't find home environment variable!");

    let mut app_dir = PathBuf::new();
    app_dir.push(home);
    app_dir.push(".cache/desktop-entry-daemon/share/");
    if clean {
        // Clear old entries (won't error if it doesn't exist)
        let _ = fs::remove_dir_all(app_dir.clone());
        // Create the desktop-entry-daemon directory
        let _ = fs::create_dir_all(app_dir.clone().join(Path::new("icons")));
        let _ = fs::create_dir_all(app_dir.clone().join(Path::new("applications")));
    }
    log::debug!("Got data dir: {:?}", app_dir);
    app_dir.to_owned()
}

pub fn set_up_environment(catalog: Arc<Mutex<EntryCatalog>>) -> Daemon {
    Daemon {
        data_dir: get_data_dir(true).into(),
        catalog,
    }
}

pub fn clean_environment() {
    let _ = get_data_dir(true);
}
