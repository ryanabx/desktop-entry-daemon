use std::error::Error;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use async_std::sync::Arc;
use async_std::sync::Mutex;
use std::fs::{self, create_dir_all};
use zbus::fdo::DBusProxy;
use zbus::message::Header;
use zbus::object_server::SignalContext;
use zbus::proxy::CacheProperties;
use zbus::{interface, proxy::Builder, Connection};

use crate::desktop_entry::validate_desktop_entry;
use crate::types::{DesktopEntry, EntryCatalog, IconEntry};

pub struct Daemon {
    pub data_dir: PathBuf,
    pub catalog: Arc<Mutex<EntryCatalog>>,
}

#[interface(name = "net.ryanabx.DesktopEntry")]
impl Daemon {
    /// Register a new application entry. The utf-8 encoded `entry` will be validated to be conformant with the
    /// [Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/)
    /// Returns an error if the entry failed to register.
    /// requires a valid process identifier to watch, entry goes away after the identified process exits
    async fn register_entry(
        &mut self,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        #[zbus(connection)] conn: &Connection,
        appid: String,
        entry: String,
    ) -> zbus::fdo::Result<()> {
        let dbus_proxy = DBusProxy::builder(conn)
            .cache_properties(CacheProperties::No)
            .build()
            .await
            .unwrap();
        let pid = dbus_proxy
            .get_connection_credentials(hdr.destination().unwrap().to_owned())
            .await
            .unwrap()
            .process_id()
            .unwrap();
        log::debug!("PID of client: {:?}", pid);
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
                        let new_entry = DesktopEntry {
                            appid,
                            path: desktop_file_path.clone(),
                        };
                        let _ = Daemon::entry_changed(&ctxt, &new_entry.appid).await;
                        self.catalog.lock().await.add_desktop_entry(pid, new_entry);
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("Could not write entry for id '{}', error: {:?}", appid, e);
                        Err(zbus::fdo::Error::IOError(
                            "Could not write entry for app id".to_string(),
                        ))
                    }
                }
            }
            Err(e) => {
                log::error!("{} failed validation! {}", appid, e);
                Err(zbus::fdo::Error::InvalidArgs(
                    "Desktop file failed validation!".to_string(),
                ))
            }
        }
    }

    /// Register a new application icon. The icon data should be valid .png or .svg data, and the icon type should be
    /// 0 for .png, 1 for .svg. The icon name is the name desktop entries reference when using the icon. The method will
    /// returns true if successful, false otherwise.
    async fn register_icon(
        &mut self,
        #[zbus(header)] hdr: Header<'_>,
        #[zbus(signal_context)] ctxt: SignalContext<'_>,
        #[zbus(connection)] conn: &Connection,
        name: String,
        data: &[u8],
    ) -> zbus::fdo::Result<()> {
        let dbus_proxy = DBusProxy::builder(conn)
            .cache_properties(CacheProperties::No)
            .build()
            .await
            .unwrap();
        let pid = dbus_proxy
            .get_connection_credentials(hdr.destination().unwrap().to_owned())
            .await
            .unwrap()
            .process_id()
            .unwrap();
        log::debug!("PID of client: {:?}", pid);
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
                return Err(zbus::fdo::Error::InvalidArgs(
                    "Image width and height must be the same!".to_string(),
                ));
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
                return Err(zbus::fdo::Error::FileExists(
                    "Icon already exists!".to_string(),
                ));
            }
            match img.save_with_format(
                self.data_dir.join(Path::new(f_path)),
                image::ImageFormat::Png,
            ) {
                Ok(_) => {
                    log::info!("Success! {}", name);
                    let new_entry = IconEntry {
                        icon_name: name,
                        icon_path: self.data_dir.join(Path::new(f_path)),
                    };
                    let _ = Daemon::icon_changed(&ctxt, &new_entry.icon_name).await;
                    self.catalog.lock().await.add_icon(pid, new_entry);
                    Ok(())
                }
                Err(e) => {
                    log::error!("Could not write png to data dir. Reason: {}", e);
                    Err(zbus::fdo::Error::IOError(format!(
                        "Could not write image to data dir. Reason: {}",
                        e
                    )))
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
                    return Err(zbus::fdo::Error::FileExists(
                        "Icon already exists!".to_string(),
                    ));
                }
                match fs::write(self.data_dir.join(Path::new(f_path)), text_data.as_bytes()) {
                    Ok(_) => {
                        log::info!("Success! {}", name);
                        let new_entry = IconEntry {
                            icon_name: name,
                            icon_path: self.data_dir.join(Path::new(f_path)),
                        };
                        let _ = Daemon::icon_changed(&ctxt, &new_entry.icon_name).await;
                        self.catalog.lock().await.add_icon(pid, new_entry);
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("Could not write svg to data dir. Reason: {}", e);
                        Err(zbus::fdo::Error::IOError(format!(
                            "Could not write image to data dir. Reason: {}",
                            e
                        )))
                    }
                }
            } else {
                log::error!("Could not convert text data to svg.");
                Err(zbus::fdo::Error::InvalidArgs(
                    "Image data given failed svg validation.".to_string(),
                ))
            }
        } else {
            Err(zbus::fdo::Error::InvalidArgs(
                "Image data given could not be converted to plaintext svg or binary image format."
                    .to_string(),
            ))
        }
    }

    #[zbus(signal)]
    /// signal for when an entry is added or destroyed. subscribe to this if you would like to manually
    /// handle refreshing the xdg desktop database, i.e. by using `update-desktop-database`
    /// this is normally handled automatically by desktop-entry-daemon
    async fn entry_changed(signal_ctxt: &SignalContext<'_>, appid: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    /// signal for when an icon is added or destroyed. subscribe to this if you would like to manually
    /// handle refreshing the xdg desktop database, i.e. by using `update-desktop-database`
    /// this is normally handled automatically by desktop-entry-daemon
    async fn icon_changed(signal_ctxt: &SignalContext<'_>, icon_name: &str) -> zbus::Result<()>;

    /// register the sender as a change handler for icons and entries. this inhibits the behavior
    /// of desktop-entry-daemon refreshing the database whenever a new icon or entry is added or
    /// removed. along with this, if you'd like to watch changes, subscribe to `icon_changed` and
    /// `entry_changed`
    async fn register_change_handler(&mut self, pid: u32) -> zbus::fdo::Result<()> {
        self.catalog.lock().await.change_handlers.insert(pid);
        Ok(())
    }
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
