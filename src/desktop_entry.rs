use std::{fs, path::Path};

use freedesktop_desktop_entry::{default_paths, DesktopEntry, Iter};

use crate::daemon::ValidationError;

/// validate a desktop entry. takes in an entry path and returns the resulting desktop
/// entry string and the application id
pub fn validate_desktop_entry(entry: &str, appid: &str) -> Result<String, ValidationError> {
    log::debug!("appid: {}", appid);
    log::trace!("entry: {}", entry);
    if let Err(e) = DesktopEntry::decode(Path::new(&format!("{}.desktop", appid)), &entry) {
        log::error!("Warning: Desktop file failed validation");
        Err(ValidationError::NotValid(e.to_string()))
    } else if app_exists(appid) {
        Err(ValidationError::DuplicateAppID)
    } else {
        Ok(entry.to_string())
    }
    // TODO: Extra validation (strip exec, etc...)
}

fn app_exists(id: &str) -> bool {
    for path in Iter::new(default_paths()) {
        // let path_src = PathSource::guess_from(&path);
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
