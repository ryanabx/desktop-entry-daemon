use std::path::Path;
use std::time::Duration;

use async_std::sync::{Arc, Mutex};

use async_std::task;
use entry_management::EntryManager;
use zbus::{Connection, Result as ZbusResult};

use crate::daemon::Daemon;
use crate::entry_management::Lifetime;
use crate::tools::get_dirs;

mod daemon;
mod entry_management;
mod tools;

#[async_std::main]
async fn main() -> ZbusResult<()> {
    env_logger::init();
    let (tmp_dir, persistent_dir, config_file) = get_dirs();
    let manager = Arc::new(Mutex::new(EntryManager::new(
        tmp_dir,
        persistent_dir,
        config_file,
    )));
    let c = manager.clone();
    let _ = async_std::task::spawn(async { watch_processes(c).await });
    let c = manager.clone();
    provide_desktop_entry_api(c).await?;
    Ok(())
}

async fn provide_desktop_entry_api(manager: Arc<Mutex<EntryManager>>) -> zbus::Result<()> {
    let daemon = Daemon {
        entry_manager: manager,
    };
    // start daemon
    let connection = Connection::session().await?;
    // setup the server
    connection
        .object_server()
        .at("/org/desktopintegration/DesktopEntry", daemon)
        .await?;
    // before requesting the name
    connection
        .request_name("org.desktopintegration.DesktopEntry")
        .await?;
    log::info!("Running server connection and listening for calls");

    loop {
        // do something else, wait forever or timeout here:
        // handling D-Bus messages is done in the background
        std::future::pending::<()>().await;
    }
}

async fn watch_processes(manager: Arc<Mutex<EntryManager>>) -> zbus::Result<()> {
    log::info!("Watching if processes exit!");
    loop {
        task::sleep(Duration::from_secs(1)).await;
        // Check if processes have been destroyed
        let mut manager_lock = manager.lock_arc().await;
        let keys_to_iter = manager_lock
            .cache
            .entries
            .keys()
            .cloned()
            .chain(manager_lock.cache.icons.keys().cloned())
            .filter_map(|x| {
                if let Lifetime::Process(pid) = x {
                    Some(pid)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        for x in keys_to_iter {
            if !Path::new(&format!("/proc/{}", x.clone())).exists() {
                log::info!("Process {} has exited! Removing associated entries...", x);
                if manager_lock.remove_lifetime(Lifetime::Process(x)).is_err() {
                    log::error!("Something went wrong when removing lifetime with PID {}", x);
                }
            }
        }
        let keys_to_iter = manager_lock
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
                manager_lock.change_handlers.remove(&x);
            }
        }
    }
}
