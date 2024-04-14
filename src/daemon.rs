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
    /// Register a new application entry. The utf-8 encoded `entry` will be validated to be conformant with the
    /// [Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/)
    /// Returns true if the operation is successful, false otherwise
    async fn register_entry(&self, app_id: String, entry: String) -> bool {
        log::debug!("Received entry for app id: {:?}", app_id);
        if let Some(entry) = validate_desktop_entry(entry) {
            let desktop_file_path = &self
                .data_dir
                .as_path()
                .join(format!("applications/{}.desktop", app_id));
            let _ = create_dir_all(self.data_dir.join(Path::new("applications")));
            match std::fs::write(desktop_file_path, entry.as_bytes()) {
                Ok(_) => {
                    log::info!("Successful entry: {}", app_id);
                    return true;
                }
                Err(e) => {
                    log::error!("Could not write entry. '{}', error: {:?}", app_id, e);
                }
            }
        }
        false
    }

    /// Register a new application icon. The icon data should be valid .png or .svg data, and the icon type should be
    /// 0 for .png, 1 for .svg. The icon name is the name desktop entries reference when using the icon. The method will
    /// returns true if successful, false otherwise.
    async fn register_icon(&self, icon_name: String, icon_data: &[u8]) -> bool {
        if let Ok(img) = image::io::Reader::new(std::io::Cursor::new(icon_data))
            .with_guessed_format()
            .unwrap()
            .decode()
        {
            log::info!("{} is a valid image", icon_name);
            let image_size_dir = format!("{}x{}", img.width(), img.height());
            let _ = create_dir_all(
                self.data_dir
                    .join(Path::new(&format!("hicolor/{}/apps/", image_size_dir))),
            );
            if let Ok(_) = fs::write(
                self.data_dir.join(Path::new(&format!(
                    "hicolor/{}/apps/{}.svg",
                    image_size_dir, icon_name
                ))),
                img.as_bytes(),
            ) {
                log::info!("Success! {}", icon_name);
                return true;
            }
        } else if let Ok(text_data) = String::from_utf8(icon_data.into()) {
            log::info!("{} is valid utf8 text", icon_name);
            if let Ok(_) = svg::read(&text_data) {
                if let Ok(_) = fs::write(
                    self.data_dir.join(Path::new(&format!(
                        "hicolor/scalable/apps/{}.svg",
                        icon_name
                    ))),
                    text_data.as_bytes(),
                ) {
                    log::info!("Success! {}", icon_name);
                    return true;
                }
            }
        }
        false
    }
}
