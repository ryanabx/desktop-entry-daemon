use async_std::path::{Path, PathBuf};
use zbus::{interface, Connection, Result as ZbusResult};

mod utils;

struct Daemon {
    data_dir: PathBuf,
}

#[interface(name = "net.ryanabx.XDGTempDaemon")]
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
    let daemon = utils::set_up_environment();
    let connection = Connection::session().await?;
    // setup the server
    connection
        .object_server()
        .at("/net/ryanabx/XDGTempDaemon", daemon)
        .await?;
    // before requesting the name
    connection.request_name("net.ryanabx.XDGTempDaemon").await?;

    loop {
        // do something else, wait forever or timeout here:
        // handling D-Bus messages is done in the background
        std::future::pending::<()>().await;
    }
}
