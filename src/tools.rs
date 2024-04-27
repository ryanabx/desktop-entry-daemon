use std::{
    env, fs,
    path::{Path, PathBuf},
};

use freedesktop_desktop_entry::{default_paths, DesktopEntry, Iter};

use crate::daemon::ValidationError;

/// validate a desktop entry. takes in an entry path and returns the resulting desktop
/// entry string and the application id
pub fn validate_desktop_entry(entry: &str, appid: &str) -> Result<String, ValidationError> {
    log::debug!("appid: {}", appid);
    log::trace!("entry: {}", entry);
    // TODO: Extra validation (strip exec, etc...)
    if let Err(e) = DesktopEntry::decode(Path::new(&format!("{}.desktop", appid)), &entry) {
        log::error!("Warning: Desktop file failed validation");
        Err(ValidationError::NotValid(e.to_string()))
    } else if app_exists(appid) {
        Err(ValidationError::DuplicateAppID)
    } else {
        Ok(entry.to_string())
    }
}

fn app_exists(id: &str) -> bool {
    for path in Iter::new(default_paths()) {
        if let Ok(bytes) = fs::read_to_string(&path) {
            if let Ok(entry) = DesktopEntry::decode(&path, &bytes) {
                if entry.appid == id {
                    return true;
                }
            }
        }
    }
    false
}

pub fn get_dirs() -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let home_str = env::var("HOME").unwrap();
    let runtime_dir = env::var("RUNTIME_DIRECTORY").unwrap_or(format!(
        "/run/user/{}/desktop-entry-daemon/",
        env::var("UID").unwrap_or("1000".to_string())
    ));
    let proc_dir = &Path::new(&runtime_dir).join("process");
    if !proc_dir.exists() {
        log::warn!(
            "proc_dir {} does not exist! creating directory...",
            proc_dir.to_str().unwrap()
        );
        fs::create_dir(proc_dir).unwrap();
    }
    let _ = fs::create_dir(proc_dir.join(Path::new("applications")));
    let _ = fs::create_dir(proc_dir.join(Path::new("icons")));

    let session_dir = &Path::new(&runtime_dir).join("session");
    if !session_dir.exists() {
        log::warn!(
            "session_dir {} does not exist! creating directory...",
            session_dir.to_str().unwrap()
        );
        fs::create_dir(session_dir).unwrap();
    }
    let _ = fs::create_dir(session_dir.join(Path::new("applications")));
    let _ = fs::create_dir(session_dir.join(Path::new("icons")));

    let persistent_dir_str = format!("{}/.cache/desktop-entry-daemon/", home_str);
    let persistent_dir = Path::new(&persistent_dir_str);
    if !persistent_dir.exists() {
        log::warn!(
            "persistent_dir {} does not exist! creating directory...",
            persistent_dir.to_str().unwrap()
        );
        fs::create_dir(persistent_dir).unwrap();
    }
    let _ = fs::create_dir(persistent_dir.join(Path::new("applications")));
    let _ = fs::create_dir(persistent_dir.join(Path::new("icons")));

    let config_file_str = format!("{}/.config/desktop-entry-daemon/cache.ron", home_str);
    let config_file = Path::new(&config_file_str);
    let _ = fs::create_dir(config_file.parent().unwrap());
    log::debug!(
        "proc_dir: {:?} | session_dir: {:?} | persistent_dir: {:?} | config_file: {:?}",
        proc_dir,
        session_dir,
        persistent_dir,
        config_file,
    );
    (
        proc_dir.to_owned(),
        session_dir.to_owned(),
        persistent_dir.to_owned(),
        config_file.to_owned(),
    )
}
