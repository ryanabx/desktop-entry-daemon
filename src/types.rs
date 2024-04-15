use std::path::PathBuf;

use zbus::names::{OwnedUniqueName, UniqueName};

#[derive(Clone, Debug)]
pub struct DesktopEntry {
    pub appid: String,
    pub path: PathBuf,
}

impl DesktopEntry {}

pub struct IconEntry {
    pub icon_name: String,
    pub icon_path: PathBuf,
}

impl IconEntry {}
