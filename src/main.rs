use async_std::{fs::File, io::WriteExt, path::PathBuf};
use tempfile::TempDir;
use zbus::{interface, Connection, Result as ZbusResult};

struct Daemon {
    path: TempDir,
    entries: Vec<PathBuf>,
}

#[interface(name = "org.ryanabx.DesktopEntryDaemon")]
impl Daemon {
    /// Register a desktop entry. Required is the `domain` name (e.g. com.ryanabx.TabletopEngine)
    /// and the plaintext `entry`
    async fn register_entry(&self, domain: &str, entry: &str) -> String {
        let file_path = self.path.path().join("my-temporary-note.txt");
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
    let daemon = Daemon {
        path: TempDir::new().expect("Could not create a temporary directory inside tmp"),
        entries: Vec::new(),
    };

    return daemon;
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
