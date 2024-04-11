use std::{fs::read_to_string, path::Path, process::Command};

/// validate a desktop entry. takes in an entry path and returns the resulting desktop
/// entry string and the application id
pub fn validate_desktop_entry(entry_path: String) -> Option<(String, String)> {
    let entry = Path::new(&entry_path);
    // Make sure path exists
    if !entry.exists() {
        return None;
    } else if let Some(false) | None =
        run_desktop_file_validate(entry.to_str().unwrap().to_string())
    {
        return None;
    }
    // TODO: Extra validation (strip exec, etc...)
    let (entry_text, app_id) = (
        read_to_string(entry).unwrap_or_default(),
        entry
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string(),
    );
    if entry_text.is_empty() || app_id.is_empty() {
        None
    } else {
        Some((entry_text, app_id))
    }
}

fn run_desktop_file_validate(entry_path: String) -> Option<bool> {
    match Command::new("sh")
        .arg("-c")
        .arg(format!("desktop-file-validate {}", entry_path))
        .output()
    {
        Ok(output) => {
            println!("Output: '{}'", String::from_utf8(output.stdout).unwrap());
            Some(true)
        }
        Err(_) => None,
    }
}
