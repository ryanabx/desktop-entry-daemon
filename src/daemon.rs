use std::error::Error;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use std::fs::{self, create_dir_all};
use zbus::interface;

use crate::desktop_entry::validate_desktop_entry;

pub struct Daemon {
    pub data_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub enum ValidationError {
    DuplicateAppID,
    NotValid(String),
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::DuplicateAppID => {
                write!(f, "Duplicate app id")
            }
            ValidationError::NotValid(reason) => {
                write!(f, "Desktop entry failed validation: {}", reason)
            }
        }
    }
}

impl Error for ValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

#[interface(name = "net.ryanabx.DesktopEntry")]
impl Daemon {
    /// Register a new application entry. The utf-8 encoded `entry` will be validated to be conformant with the
    /// [Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/)
    /// Returns true if the operation is successful, false otherwise
    fn register_entry(&self, appid: String, entry: String) -> bool {
        log::debug!("Received entry for app id: {:?}", appid);
        match validate_desktop_entry(&entry, &appid) {
            Ok(entry) => {
                let desktop_file_path = &self
                    .data_dir
                    .as_path()
                    .join(format!("applications/{}.desktop", appid));
                let _ = create_dir_all(self.data_dir.join(Path::new("applications")));
                match std::fs::write(desktop_file_path, entry.as_bytes()) {
                    Ok(_) => {
                        log::info!("Successful entry: {}", appid);
                        true
                    }
                    Err(e) => {
                        log::error!("Could not write entry for id '{}', error: {:?}", appid, e);
                        false
                    }
                }
            }
            Err(e) => {
                log::error!("{} failed validation! {}", appid, e);
                false
            }
        }
    }

    /// Register a new application icon. The icon data should be valid .png or .svg data, and the icon type should be
    /// 0 for .png, 1 for .svg. The icon name is the name desktop entries reference when using the icon. The method will
    /// returns true if successful, false otherwise.
    fn register_icon(&self, name: String, data: &[u8]) -> bool {
        if let Ok(img) = image::io::Reader::new(std::io::Cursor::new(data))
            .with_guessed_format()
            .unwrap()
            .decode()
        {
            log::info!("{} is a valid image", name);
            if img.width() != img.height() {
                log::error!(
                    "Image width and height are different! {} != {}",
                    img.width(),
                    img.height()
                );
                return false;
            }
            let img = if img.width() > 512 {
                log::warn!("Image size was greater than 512! Resizing icon to 512x512.");
                img.resize(512, 512, image::imageops::FilterType::Lanczos3)
            } else {
                img
            };
            let image_size_dir = format!("{}x{}", img.width(), img.height());
            let _ = create_dir_all(self.data_dir.join(Path::new(&format!(
                "icons/hicolor/{}/apps/",
                image_size_dir
            ))));
            let f_path = &format!("icons/hicolor/{}/apps/{}.png", image_size_dir, name);
            if self.data_dir.join(Path::new(f_path)).exists() {
                log::error!(
                    "Path '{}' already exists!",
                    self.data_dir.join(Path::new(f_path)).to_str().unwrap()
                );
                return false;
            }
            match img.save_with_format(
                self.data_dir.join(Path::new(f_path)),
                image::ImageFormat::Png,
            ) {
                Ok(_) => {
                    log::info!("Success! {}", name);
                    true
                }
                Err(e) => {
                    log::error!("Could not write png to data dir. Reason: {}", e);
                    false
                }
            }
        } else if let Ok(text_data) = String::from_utf8(data.into()) {
            log::info!("{} is valid utf8 text", name);
            if let Ok(_) = svg::read(&text_data) {
                let _ = create_dir_all(
                    self.data_dir
                        .join(Path::new("icons/hicolor/scalable/apps/")),
                );
                let f_path = &format!("icons/hicolor/scalable/apps/{}.svg", name);
                if self.data_dir.join(Path::new(f_path)).exists() {
                    log::error!(
                        "Path '{}' already exists!",
                        self.data_dir.join(Path::new(f_path)).to_str().unwrap()
                    );
                    return false;
                }
                match fs::write(self.data_dir.join(Path::new(f_path)), text_data.as_bytes()) {
                    Ok(_) => {
                        log::info!("Success! {}", name);
                        true
                    }
                    Err(e) => {
                        log::error!("Could not write svg to data dir. Reason: {}", e);
                        false
                    }
                }
            } else {
                log::error!("Could not convert text data to svg.");
                false
            }
        } else {
            false
        }
    }
}
