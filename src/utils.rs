use std::fs;
use std::path::{Path, PathBuf};

use xdg::BaseDirectories;

use crate::Daemon;

pub fn copy<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), std::io::Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        println!("process: {:?}", &working_path);

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            println!(" mkdir: {:?}", dest);
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
                        println!("  copy: {:?} -> {:?}", &path, &dest_path);
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        println!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn validate_data<U: AsRef<Path>>(from: U) -> bool {
    true // TODO: Data validation
}

fn add_path_to_env_if_not_exists(variable: String, val: String) {
    let v = std::env::var(&variable).unwrap_or("".to_string());
    if !v.split(":").any(|x| Path::new(x) == Path::new(&val)) {
        println!("Adding variable");
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "export XDG_DATA_DIRS && XDG_DATA_DIRS=\"$XDG_DATA_DIRS:{}\"",
                val
            ))
            .output()
            .expect("could not run command");
        std::env::set_var(&variable, format!("{}:{}", v, &val));
        println!("{}", std::env::var(&variable).unwrap_or("".to_string()));
    }
}

fn get_data_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        let mut path = PathBuf::new();
        path.push(&home);
        path.push(".cache/xdg-temp-daemon/share/");
        add_path_to_env_if_not_exists("XDG_DATA_DIRS".to_string(), path.to_str().unwrap().into());
    }
    let base_dir = BaseDirectories::new().expect("could not get XDG base directories");
    let data_dirs = base_dir.get_data_dirs();
    // Find the xdg-temp-daemon directory
    let app_dir = data_dirs
        .iter()
        .find(|x| {
            x.ends_with(Path::new(".cache/xdg-temp-daemon/share/"))
        })
        .expect("cannot find xdg-temp-daemon xdg data directory");
    // Clear old entries (won't error if it doesn't exist)
    let _ = fs::remove_dir_all(app_dir.clone());
    // Create the xdg-temp-daemon directory
    let _ = fs::create_dir_all(app_dir.clone()).expect("could not create directory");
    app_dir.to_owned()
}

pub fn set_up_environment() -> Daemon {
    let data_dir = get_data_dir();
    Daemon {
        data_dir: data_dir.clone().into(),
    }
}
