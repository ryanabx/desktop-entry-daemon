use async_std::{
    fs::{self, File},
    io::WriteExt,
    path::{Path, PathBuf},
};
use xdg::BaseDirectories;
use zbus::{interface, Connection, Result as ZbusResult};
use std::process::Command;

mod utils;

struct Daemon {
    data_dir: PathBuf,
    files: Vec<tempfile::NamedTempFile>
}

#[interface(name = "net.ryanabx.XDGTempDaemon")]
impl Daemon {
    /// Register temporary XDG data. Requires the `path` to the data directory to copy.
    /// The data can include desktop entries, icons, etc.
    /// Entries are cleared when the daemon restarts
    async fn temp_data(&self, path: &str) -> bool {
        let data_path = Path::new(path);


        true
    }
}

async fn set_up_environment() -> Daemon {
    let base_dir = BaseDirectories::new().expect("could not get XDG base directories");
    // Find the xdg-temp-daemon directory
    let app_dir = base_dir
        .get_data_dirs()
        .iter()
        .find(|x| {
            println!("{:?}", x);
            x.ends_with(Path::new("xdg-temp-daemon/share"))
        })
        .expect("cannot find xdg-temp-daemon xdg data directory");
    // Clear old entries (won't error if it doesn't exist)
    let _ = fs::remove_dir_all(app_dir.clone());
    // Create the xdg-temp-daemon directory
    let _ = fs::create_dir_all(app_dir.clone())
        .await
        .expect("could not create directory");
    Daemon {
        data_dir: app_dir.clone().into(),
        files: Vec::new()
    }
}

#[async_std::main]
async fn main() -> ZbusResult<()> {
    let daemon = set_up_environment().await;
    let connection = Connection::session().await?;
    // setup the server
    connection
        .object_server()
        .at("/net/ryanabx/XDGTempDaemon", daemon)
        .await?;
    // before requesting the name
    connection
        .request_name("net.ryanabx.XDGTempDaemon")
        .await?;

    loop {
        // do something else, wait forever or timeout here:
        // handling D-Bus messages is done in the background
        std::future::pending::<()>().await;
    }
}
