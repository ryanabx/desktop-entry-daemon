use std::env;
use std::fs::{create_dir, remove_dir_all};
use std::path::{Path, PathBuf};
use std::time::Duration;

use async_std::sync::Arc;

use async_std::sync::Mutex;
use async_std::task;
use daemon::Daemon;
use zbus::{Connection, Result as ZbusResult};

use crate::types::EntryCatalog;

mod daemon;
mod desktop_entry;
mod types;

#[async_std::main]
async fn main() -> ZbusResult<()> {
    env_logger::init();
    let catalog = Arc::new(Mutex::new(EntryCatalog::new()));
    let c = catalog.clone();
    let _ = async_std::task::spawn(async { watch_proc(c).await });
    let c = catalog.clone();
    provide_desktop_entry_api(c).await?;
    Ok(())
}

async fn provide_desktop_entry_api(catalog: Arc<Mutex<EntryCatalog>>) -> zbus::Result<()> {
    let daemon = set_up_environment(catalog);
    // start daemon
    let connection = Connection::session().await?;
    // setup the server
    connection
        .object_server()
        .at("/net/ryanabx/DesktopEntry", daemon)
        .await?;
    // before requesting the name
    connection.request_name("net.ryanabx.DesktopEntry").await?;
    log::info!("Running server connection and listening for calls");

    loop {
        // do something else, wait forever or timeout here:
        // handling D-Bus messages is done in the background
        std::future::pending::<()>().await;
    }
}

async fn watch_proc(catalog: Arc<Mutex<EntryCatalog>>) -> zbus::Result<()> {
    log::info!("Watching if processes exit!");
    loop {
        task::sleep(Duration::from_secs(10)).await;
        // Check if processes have been destroyed
        let mut catalog_lock = catalog.lock_arc().await;
        let keys_to_iter = catalog_lock
            .owned_resources
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for x in keys_to_iter {
            if !Path::new(&format!("/proc/{}", x.clone())).exists() {
                log::info!("Process {} has exited! Removing associated entries...", x);
                catalog_lock.remove_owner(x);
            }
        }
        let keys_to_iter = catalog_lock
            .change_handlers
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        for x in keys_to_iter {
            if !Path::new(&format!("/proc/{}", x)).exists() {
                log::info!(
                    "Process {} has exited! Removing associated change handler...",
                    x
                );
                catalog_lock.change_handlers.remove(&x);
            }
        }
    }
}

pub fn get_data_dir(clean: bool) -> PathBuf {
    let home = match env::var("RUNTIME_DIRECTORY") {
        Ok(h) => h,
        Err(_) => {
            log::error!("RUNTIME_DIRECTORY NOT FOUND. Make sure you're using the service!");
            panic!()
        }
    };

    let app_dir = Path::new(&home);
    if !app_dir.exists() {
        log::warn!(
            "Runtime directory {} does not exist! Attempting to create directory manually...",
            app_dir.to_str().unwrap()
        );
        match create_dir(app_dir) {
            Ok(_) => {
                log::info!("App directory created!");
            }
            Err(e) => {
                log::error!("App directory could not be created. Reason: {}", e);
                panic!("App directory could not be created");
            }
        }
    }
    if clean {
        // Clear old entries (won't error if it doesn't exist)
        let _ = remove_dir_all(app_dir.join(Path::new("applications")));
        let _ = remove_dir_all(app_dir.join(Path::new("icons")));
    }
    let _ = create_dir(app_dir.join(Path::new("applications")));
    let _ = create_dir(app_dir.join(Path::new("icons")));
    log::debug!("Got data dir: {:?}", app_dir);
    app_dir.to_owned()
}

pub fn set_up_environment(catalog: Arc<Mutex<EntryCatalog>>) -> Daemon {
    Daemon {
        data_dir: get_data_dir(false).into(),
        catalog,
    }
}
