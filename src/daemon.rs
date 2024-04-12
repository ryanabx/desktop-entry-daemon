use std::io::Write;
use std::path::PathBuf;

use std::fs::{remove_file, File};
use zbus::interface;

use crate::freedesktop::desktop_entry::validate_desktop_entry;

pub struct Daemon {
    pub data_dir: PathBuf,
}

#[interface(name = "net.ryanabx.DesktopEntry")]
impl Daemon {
    /// register desktop application entries. each entry path should be encoded in a string in the list, and
    /// should follow the [desktop entry spec](https://specifications.freedesktop.org/desktop-entry-spec/latest/)
    async fn register_entries(&self, entry_paths: Vec<&str>) -> Vec<String> {
        log::debug!("Received entries: {:?}", entry_paths);
        let modified_entries = entry_paths
            .iter()
            .filter_map(|entry| validate_desktop_entry(entry))
            .collect::<Vec<_>>();
        let mut successful_entries = Vec::new();
        for (ref entry, app_id) in modified_entries {
            let desktop_file_path = &self
                .data_dir
                .as_path()
                .join(format!("applications/{}.desktop", app_id));
            if let Ok(mut f) = File::create(desktop_file_path) {
                if let Ok(_) = f.write_all(entry.as_bytes()) {
                    successful_entries.push(app_id);
                }
            }
        }
        successful_entries
    }

    /// register icons for applications. each icon must follow the
    /// [icon theme spec](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html)
    async fn register_icons(&self, icon_paths: Vec<&str>) -> Vec<String> {
        log::debug!("Received icons: {:?}", icon_paths);
        Vec::new()
    }

    /// remove desktop application entries. use the app_id to reference the entry
    /// to delete from the desktop-entry-daemon data directory
    async fn remove_entries(&self, entry_names: Vec<&str>) -> Vec<String> {
        log::debug!("Received entries to remove: {:?}", entry_names);
        let mut successful_entries = Vec::new();
        for app_id in entry_names {
            if let Ok(_) = remove_file(
                self.data_dir
                    .as_path()
                    .join(format!("applications/{}.desktop", app_id)),
            ) {
                successful_entries.push(app_id.to_string());
            }
        }
        successful_entries
    }

    /// remove icons. use the icon name to reference the entry
    /// to delete from the desktop-entry-daemon data directory
    async fn remove_icons(&self, icon_names: Vec<&str>) -> Vec<String> {
        log::debug!("Received icons to remove: {:?}", icon_names);
        Vec::new()
    }
}
