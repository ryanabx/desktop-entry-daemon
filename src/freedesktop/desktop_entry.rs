/// validate a desktop entry. takes in an entry path and returns the resulting desktop
/// entry string and the application id
pub fn validate_desktop_entry(entry_path: String) -> Option<(String, String)> {
    // TODO: Implement
    // Write file to disk (tmpfile)
    // Run validate-desktop-entry
    // If that comes back clean, do extra validation (strip exec, etc...)
    // Return validated entry
    Some(("".into(), "".into()))
}
