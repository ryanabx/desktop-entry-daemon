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
    async fn register_entries(&self, entry_paths: Vec<String>) -> bool {
        let mut success = true;
        let modified_entries = entry_paths
            .iter()
            .map(|entry| {
                validate_desktop_entry(entry.clone()).unwrap_or_else(|| {
                    success = false;
                    ("".into(), "".into())
                })
            })
            .collect::<Vec<_>>();
        if !success {
            return false; // question: should we always early exit for this?
        }
        for (ref entry, app_id) in modified_entries {
            let desktop_file_path = &self
                .data_dir
                .as_path()
                .join(format!("applications/{}.desktop", app_id));
            if let Ok(mut f) = File::create(desktop_file_path) {
                if let Err(_) = f.write_fmt(format_args!("{}", entry)) {
                    success = false;
                }
                if let Err(_) = f.flush() {
                    success = false;
                }
            } else {
                success = false;
            }
        }
        success
    }

    /// register icons for applications. each icon must follow the
    /// [icon theme spec](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html)
    async fn register_icons(&self, icon_paths: Vec<String>) -> bool {
        false
    }

    /// remove desktop application entries. use the app_id to reference the entry
    /// to delete from the desktop-entry-daemon data directory
    async fn remove_entries(&self, entry_names: Vec<String>) -> bool {
        let mut success = true;
        for app_id in entry_names {
            if let Err(_) = remove_file(
                self.data_dir
                    .as_path()
                    .join(format!("applications/{}.desktop", app_id)),
            ) {
                success = false;
            }
        }
        success
    }

    /// remove icons. use the icon name to reference the entry
    /// to delete from the desktop-entry-daemon data directory
    async fn remove_icons(&self, icon_names: Vec<String>) -> bool {
        false
    }
}
