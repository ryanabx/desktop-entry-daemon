use async_std::{fs::File, io::WriteExt, path::PathBuf};
use tempfile::TempDir;
use zbus::{interface, Error as ZbusError, Connection, DBusError, Result};

struct Daemon {
    path: TempDir,
    entries: Vec<PathBuf>,
}

#[interface(name = "org.ryanabx.DesktopEntryDaemon")]
impl Daemon {
    /// Register a desktop entry. Required is the `domain` name (e.g. com.ryanabx.TabletopEngine)
    /// and the plaintext `entry`.
    async fn register_entry(&self, domain: &str, entry: &str) -> Result<String> {
        let file_path = self.path.path().join("my-temporary-note.txt");
        if let Ok(mut desktop_file) = File::create(file_path).await {
            if writeln!(desktop_file, "{}", entry).await.is_err() {
                return Err(ZbusError::ExcessData);
            } else {
                return Ok("".to_string());
            }
        } else {
            Err(ZbusError::ExcessData)
        }
    }
    /// Destroy a previously created desktop entry.
    async fn destroy_entry(&self, identity: &str) -> bool {
        // TODO: Implement
        true
    }
}

fn set_up_environment() {}

#[async_std::main]
async fn main() -> Result<()> {
    let connection = Connection::session().await?;
    let daemon = Daemon {
        path: TempDir::new().expect("Could not create a temporary directory inside tmp"),
        entries: Vec::new(),
    };
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
