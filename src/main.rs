use clap::Parser;
use zbus::{Connection, Result as ZbusResult};

use crate::files::{clean_environment, set_up_environment};

mod daemon;
mod desktop_entry;
mod files;

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
    // start daemon
    let daemon = set_up_environment();
    let connection = Connection::session().await?;
    // setup the server
    connection
        .object_server()
        .at("/net/ryanabx/DesktopEntry", daemon)
        .await?;
    // before requesting the name
    connection.request_name("net.ryanabx.DesktopEntry").await?;

    loop {
        // do something else, wait forever or timeout here:
        // handling D-Bus messages is done in the background
        std::future::pending::<()>().await;
    }
}
