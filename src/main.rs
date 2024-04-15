use async_std::sync::Arc;

use async_std::stream::StreamExt;
use async_std::sync::Mutex;
use clap::Parser;
use daemon::Daemon;
use zbus::fdo::{DBusProxy, NameOwnerChangedArgs};
use zbus::names::OwnedUniqueName;
use zbus::{Connection, Result as ZbusResult};

use crate::files::{clean_environment, set_up_environment};
use crate::types::EntryCatalog;

mod daemon;
mod desktop_entry;
mod files;
mod types;

/// program to manage temporary desktop entries
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// clear all temporary desktop entries
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    clean: bool,
}

#[async_std::main]
async fn main() -> ZbusResult<()> {
    env_logger::init();
    let args = Args::parse();
    if args.clean {
        // clean space
        clean_environment();
        return Ok(());
    }
    let catalog = Arc::new(Mutex::new(EntryCatalog::new()));
    let c = catalog.clone();
    let _ = async_std::task::spawn(async { watch_name_owner_changed(c).await });
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

async fn watch_name_owner_changed(catalog: Arc<Mutex<EntryCatalog>>) -> zbus::Result<()> {
    log::info!("Watching if name owner changes!");
    let connection = Connection::system().await?;
    // `Systemd1ManagerProxy` is generated from `Systemd1Manager` trait
    let dbus_proxy = DBusProxy::new(&connection).await?;
    // Method `receive_job_new` is generated from `job_new` signal
    let mut name_owner_changed_stream = dbus_proxy.receive_name_owner_changed().await?;

    while let Some(msg) = name_owner_changed_stream.next().await {
        // struct `JobNewArgs` is generated from `job_new` signal function arguments
        let args: NameOwnerChangedArgs = msg.args().expect("Error parsing message");

        log::info!(
            "NameOwnerChanged received: name={} old_owner={:?} new_owner={:?}",
            args.name(),
            args.old_owner(),
            args.new_owner()
        );

        log::info!("{:?}", catalog.lock().await);
        if args.new_owner().is_none() && args.old_owner().is_some() {
            catalog.lock().await.remove_owner(OwnedUniqueName::from(
                args.old_owner().as_ref().unwrap().clone(),
            ));
        }
    }

    panic!("Stream ended unexpectedly");
}
