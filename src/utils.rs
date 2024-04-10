use log::{self, log_enabled};
use std::path::{Path, PathBuf};
use std::{env, fs};

use xdg::BaseDirectories;

use crate::Daemon;

pub fn copy<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), std::io::Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    if log_enabled!(log::Level::Trace) {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "tree {}",
                PathBuf::from(from.as_ref()).to_str().unwrap_or("")
            ))
            .output();
        if let Ok(o) = output {
            log::trace!("Tree: {}", String::from_utf8(o.stdout).unwrap_or_default());
        }
    }

    while let Some(working_path) = stack.pop() {
        log::trace!("process: {:?}", &working_path);

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            log::warn!(" mkdir: {:?}", dest);
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        log::trace!("  copy: {:?} -> {:?}", &path, &dest_path);
                        if let Err(_) = fs::copy(&path, &dest_path) {
                            log::warn!("skipping {} as the file was not found", &path.display());
                        }
                    }
                    None => {
                        log::warn!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    Ok(())
}

// pub fn validate_data<U: AsRef<Path>>(from: U) -> bool {
//     true // TODO: Data validation
// }

fn get_data_dir() -> PathBuf {
    let base_dir = BaseDirectories::new().expect("could not get XDG base directories");

    let home = env::var("HOME").expect("can't find home environment variable!");

    let mut app_dir = PathBuf::new();
    app_dir.push(home);
    app_dir.push(".cache/xdg-temp-daemon/share/");

    // Clear old entries (won't error if it doesn't exist)
    let _ = fs::remove_dir_all(app_dir.clone());
    // Create the xdg-temp-daemon directory
    let _ = fs::create_dir_all(app_dir.clone());
    app_dir.to_owned()
}

pub fn set_up_environment() -> Daemon {
    let data_dir = get_data_dir();
    Daemon {
        data_dir: data_dir.clone().into(),
    }
}

pub fn clean_environment() {
    let _ = get_data_dir();
}
