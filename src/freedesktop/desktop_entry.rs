use std::{fs::read_to_string, path::Path, process::Command};

/// validate a desktop entry. takes in an entry path and returns the resulting desktop
/// entry string and the application id
pub fn validate_desktop_entry(entry_path: &str) -> Option<(String, String)> {
    log::debug!("entry_path: {}", entry_path);
    let entry = Path::new(entry_path);
    // Make sure path exists
    if !entry.exists() {
        log::warn!("Warning: Path doesn't exist {}", entry_path);
        return None;
    } else if let Some(false) | None = run_desktop_file_validate(entry.to_str().unwrap()) {
        log::warn!("Warning: Desktop file failed validation");
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
        log::warn!("Warning: One is empty: ('{}', '{}')", entry_text, app_id);
        None
    } else {
        log::debug!("({}, {})", entry_text, app_id);
        Some((entry_text, app_id))
    }
}

fn run_desktop_file_validate(entry_path: &str) -> Option<bool> {
    match Command::new("sh")
        .arg("-c")
        .arg(format!(
            "desktop-file-validate --no-hints --no-warn-deprecated {}",
            entry_path
        ))
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();
            log::debug!(
                "stdout: '{}', stderr: '{}' stdout.is_empty(): {}",
                stdout,
                stderr,
                stdout.is_empty()
            );
            Some(stdout.is_empty())
        }
        Err(_) => None,
    }
}
