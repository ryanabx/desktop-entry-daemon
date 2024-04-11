use crate::daemon::Daemon;

pub mod files;

pub fn set_up_environment() -> Daemon {
    Daemon {
        data_dir: files::get_data_dir(true).into(),
    }
}

pub fn clean_environment() {
    let _ = files::get_data_dir(true);
}
