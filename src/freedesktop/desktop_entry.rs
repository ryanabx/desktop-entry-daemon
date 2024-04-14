use std::{fs::read_to_string, path::Path, process::Command};

/// validate a desktop entry. takes in an entry path and returns the resulting desktop
/// entry string and the application id
pub fn validate_desktop_entry(entry: String) -> Option<String> {
    log::debug!("entry: {}", entry);
    if let Some(false) | None = run_desktop_file_validate(&entry) {
        log::warn!("Warning: Desktop file failed validation");
        return None;
    }
    // TODO: Extra validation (strip exec, etc...)
    Some(entry)
}

fn run_desktop_file_validate(entry: &str) -> Option<bool> {
    Some(true)
    // match Command::new("sh")
    //     .arg("-c")
    //     .arg(format!(
    //         "desktop-file-validate --no-hints --no-warn-deprecated {}",
    //         entry_path
    //     ))
    //     .output()
    // {
    //     Ok(output) => {
    //         let stdout = String::from_utf8(output.stdout).unwrap();
    //         let stderr = String::from_utf8(output.stderr).unwrap();
    //         log::debug!(
    //             "stdout: '{}', stderr: '{}' stdout.is_empty(): {}",
    //             stdout,
    //             stderr,
    //             stdout.is_empty()
    //         );
    //         Some(stdout.is_empty())
    //     }
    //     Err(_) => None,
    // }
}
