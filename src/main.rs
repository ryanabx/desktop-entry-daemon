use async_std::{
    fs::{self, File},
    io::WriteExt,
    path::{Path, PathBuf},
};
use xdg::BaseDirectories;
use zbus::{interface, Connection, Result as ZbusResult};

struct Daemon {
    path: PathBuf,
}

#[interface(name = "org.ryanabx.DesktopEntryDaemon")]
impl Daemon {
    /// Register a desktop entry. Required is the `domain` name (e.g. com.ryanabx.TabletopEngine)
    /// and the plaintext `entry`. Entries are cleared when the daemon exits.
    async fn register_entry(&self, domain: &str, entry: &str) -> String {
        let file_path = self.path.join(format!("{}.desktop", domain.to_string()));
        match File::create(file_path).await {
            Ok(mut x) => match x.write(entry.as_bytes()).await {
                Ok(_) => domain.to_string(),
                Err(_) => "".into(),
            },
            Err(_) => "".into(),
        }
    }
}

fn set_up_environment() -> Daemon {
    let base_dir = BaseDirectories::new().expect("could not get XDG base directories");
    // Find the desktop-entry-daemon directory
    let app_dir = base_dir
        .get_data_dirs()
        .iter()
        .find(|x| x.ends_with(Path::new("desktop-entry-daemon")))
        .expect("cannot find desktop-entry-daemon xdg data directory")
        .join(Path::new("share/applications"));
    // Create the desktop-entry-daemon directory if it doesn't exist
    let _ = fs::create_dir(app_dir.clone());
    Daemon {
        path: app_dir.clone().into(),
    }
}

#[async_std::main]
async fn main() -> ZbusResult<()> {
    let daemon = set_up_environment();
    let connection = Connection::session().await?;
    // setup the server
    connection
        .object_server()
        .at("/org/ryanabx/DesktopEntryDaemon", daemon)
        .await?;
    // before requesting the name
    connection
        .request_name("org.ryanabx.DesktopEntryDaemon")
        .await?;

    loop {
        // do something else, wait forever or timeout here:
        // handling D-Bus messages is done in the background
        std::future::pending::<()>().await;
    }
}
