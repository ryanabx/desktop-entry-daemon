use std::collections::HashSet;

use async_std::path::PathBuf;
use tempfile::TempDir;
use zbus::{interface, Connection, Result};

struct DesktopDir {
    path: TempDir,
    entries: Vec<PathBuf>
}

struct Daemon;

#[interface(name = "org.ryanabx.DesktopEntryDaemon")]
impl Daemon {
    /// Register a desktop entry. Required is the `domain` name (e.g. com.ryanabx.TabletopEngine)
    /// and the plaintext `entry`.
    async fn register_entry(&self, domain: &str, entry: &str) -> String {
        // TODO: Implement
        "".to_string()
    }
    /// Destroy a previously created desktop entry. 
    async fn destroy_entry(&self, identity: &str) -> bool {
        // TODO: Implement
        true
    }
}

#[async_std::main]
async fn main() -> Result<()> {
    let connection = Connection::session().await?;
    // setup the server
    connection
        .object_server()
        .at("/org/ryanabx/DesktopEntryDaemon", Daemon)
        .await?;
    // before requesting the name
    connection.request_name("org.ryanabx.DesktopEntryDaemon").await?;

    loop {
        // do something else, wait forever or timeout here:
        // handling D-Bus messages is done in the background
        std::future::pending::<()>().await;
    }
}
