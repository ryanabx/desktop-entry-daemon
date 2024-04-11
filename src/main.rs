use async_std::path::{Path, PathBuf};
use clap::Parser;
use zbus::{interface, Connection, Result as ZbusResult};

mod utils;

/// program to manage temporary desktop entries
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// clear all temporary desktop entries
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    clean: bool,
}

struct Daemon {
    data_dir: PathBuf,
}

#[interface(name = "net.ryanabx.DesktopEntry")]
impl Daemon {
    /// Register XDG data. Requires the `path` to the data directory to copy.
    /// The data can include desktop entries, icons, etc.
    /// Entries are cleared when the daemon restarts
    async fn temp_data(&self, path: &str) -> bool {
        dbg!(path);
        let data_path = Path::new(path);
        if data_path.exists().await {
            // TODO: validation of input
            match utils::copy(data_path, &self.data_dir) {
                Ok(_) => {
                    return true;
                }
                Err(ex) => {
                    log::error!("Error occurred: {:?}", ex);
                    return false;
                }
            }
        }
        log::warn!("path {:?} does not exist", data_path);
        false
    }
}

#[async_std::main]
async fn main() -> ZbusResult<()> {
    env_logger::init();
    let args = Args::parse();
    if args.clean {
        // clean space
        utils::clean_environment();
        return Ok(());
    }
    // start daemon
    let daemon = utils::set_up_environment();
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
