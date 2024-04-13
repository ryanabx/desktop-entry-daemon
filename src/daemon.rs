use std::io::Write;
use std::path::{Path, PathBuf};

use std::fs::{self, create_dir_all, remove_file, File};
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
            match std::fs::write(desktop_file_path, entry.as_bytes()) {
                Ok(_) => {
                    log::info!("Successful entry: {}", app_id);
                    successful_entries.push(app_id);
                }
                Err(e) => {
                    log::error!("Could not write entry. '{}', error: {:?}", app_id, e);
                }
            }
        }
        successful_entries
    }

    /// register icons for applications. each icon must follow the
    /// [icon theme spec](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html)
    async fn register_icons(&self, icon_paths: Vec<&str>, subpaths: Vec<&str>) -> Vec<String> {
        log::debug!(
            "Received icons: {:?} and subpaths: {:?}",
            icon_paths,
            subpaths
        );
        if icon_paths.len() != subpaths.len() {
            log::error!(
                "Lengths are not the same! {} vs {}",
                icon_paths.len(),
                subpaths.len()
            );
        }
        let mut successful_icons = Vec::new();
        for (icon_path, subpath) in Iterator::zip(icon_paths.iter(), subpaths.iter()) {
            let src_path = Path::new(icon_path);
            let dst_dir = self
                .data_dir
                .as_path()
                .join("icons/")
                .as_path()
                .join(subpath);
            if !src_path.exists() {
                log::warn!("Source path for this icon does not exist! {:?}", src_path);
                continue;
            }
            if dst_dir.starts_with(&self.data_dir) {
                if !dst_dir.exists() {
                    let _ = create_dir_all(&dst_dir);
                }
                let file_name = src_path.file_name().unwrap().to_str().unwrap();
                let dst_path = dst_dir.join(Path::new(file_name));
                match fs::copy(src_path, &dst_path) {
                    Ok(_) => {
                        log::info!("Copied icon! {:?}", dst_path);
                        successful_icons.push(file_name.to_string());
                    }
                    Err(e) => {
                        log::error!("Problem copying file to '{:?}' error: {:?}", dst_path, e);
                    }
                }
            } else {
                log::warn!(
                    "dst_dir is not a subdirectory of data dir! {:?}",
                    dst_dir.canonicalize()
                );
            }
        }
        successful_icons
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
