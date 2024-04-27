use std::error::Error;
use std::fmt::Display;

use async_std::sync::Arc;
use async_std::sync::Mutex;
use zbus::fdo::DBusProxy;
use zbus::message::Header;
use zbus::names::BusName;
use zbus::proxy::CacheProperties;
use zbus::{interface, Connection};

use crate::entry_management::{EntryManager, EntryManagerError, Lifetime};

pub struct Daemon {
    pub entry_manager: Arc<Mutex<EntryManager>>,
}

impl Into<zbus::fdo::Error> for EntryManagerError {
    fn into(self) -> zbus::fdo::Error {
        match self {
            Self::EntryValidation(e) => zbus::fdo::Error::InvalidArgs(e.to_string()),
            Self::IO(e) => zbus::fdo::Error::IOError(e.to_string()),
            Self::IconValidation(e) => zbus::fdo::Error::InvalidArgs(e.to_string()),
            Self::PathCollision(p) => zbus::fdo::Error::FileExists(p.display().to_string()),
            Self::Ron(r) => zbus::fdo::Error::IOError(r.to_string()),
        }
    }
}

#[interface(name = "org.desktopintegration.DesktopEntry")]
impl Daemon {
    /// register a new desktop entry with the calling process' lifetime
    async fn new_process_entry(
        &mut self,
        #[zbus(header)] hdr: Header<'_>,
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
            .get_connection_credentials(BusName::Unique(hdr.sender().unwrap().to_owned()))
            .await
            .unwrap()
            .process_id()
            .unwrap();
        log::debug!("appid: {:?}, PID: {:?}", appid, pid);
        let lifetime = Lifetime::from_pid(pid).unwrap();
        match self
            .entry_manager
            .lock()
            .await
            .register_entry(&entry, &appid, lifetime)
        {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("{:?}", e);
                Err(e.into())
            }
        }
    }

    /// register a new desktop entry with the session's lifetime
    async fn new_session_entry(&mut self, appid: String, entry: String) -> zbus::fdo::Result<()> {
        log::debug!("appid: {:?}, session", appid);
        let lifetime = Lifetime::Session;
        match self
            .entry_manager
            .lock()
            .await
            .register_entry(&entry, &appid, lifetime)
        {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("{:?}", e);
                Err(e.into())
            }
        }
    }

    /// register a new persistent desktop entry
    async fn new_persistent_entry(
        &mut self,
        appid: String,
        entry: String,
    ) -> zbus::fdo::Result<()> {
        log::debug!("appid: {:?}, persistent", appid);
        let lifetime = Lifetime::Persistent;
        match self
            .entry_manager
            .lock()
            .await
            .register_entry(&entry, &appid, lifetime)
        {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("{:?}", e);
                Err(e.into())
            }
        }
    }

    /// register a new icon entry with the calling process' lifetime
    async fn new_process_icon(
        &mut self,
        #[zbus(header)] hdr: Header<'_>,
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
            .get_connection_credentials(BusName::Unique(hdr.sender().unwrap().to_owned()))
            .await
            .unwrap()
            .process_id()
            .unwrap();
        log::debug!("icon: {:?}, PID: {:?}", name, pid);
        let lifetime = Lifetime::from_pid(pid).unwrap();
        match self
            .entry_manager
            .lock()
            .await
            .register_icon(&name, &data, lifetime)
        {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("{:?}", e);
                Err(e.into())
            }
        }
    }

    /// register a new icon entry with the session's lifetime
    async fn new_session_icon(&mut self, name: String, data: &[u8]) -> zbus::fdo::Result<()> {
        log::debug!("icon: {:?}, session", name);
        let lifetime = Lifetime::Session;
        match self
            .entry_manager
            .lock()
            .await
            .register_icon(&name, &data, lifetime)
        {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("{:?}", e);
                Err(e.into())
            }
        }
    }

    /// register a new persistent icon entry
    async fn new_persistent_icon(&mut self, name: String, data: &[u8]) -> zbus::fdo::Result<()> {
        log::debug!("icon: {:?}, persistent", name);
        let lifetime = Lifetime::Persistent;
        match self
            .entry_manager
            .lock()
            .await
            .register_icon(&name, &data, lifetime)
        {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("{:?}", e);
                Err(e.into())
            }
        }
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
