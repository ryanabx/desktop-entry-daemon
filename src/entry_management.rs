use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::{self, create_dir_all},
    hash::Hash,
    path::{Path, PathBuf},
};

use image::{DynamicImage, ImageError};
use ron::de::SpannedError;
use serde::{Deserialize, Serialize};

use crate::{daemon::ValidationError, tools::validate_desktop_entry};

#[derive(Debug)]
pub enum EntryManagerError {
    IO(std::io::Error),
    EntryValidation(ValidationError),
    IconValidation(IconValidationError),
    PathCollision(PathBuf),
    Ron(ron::Error),
}

#[derive(Debug)]
pub enum IconValidationError {
    ImageFormat(ImageError),
    NotSquare,
    NoTypeFound,
}

impl Display for IconValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IconValidationError::ImageFormat(e) => {
                write!(f, "{}", e.to_string())
            }
            IconValidationError::NoTypeFound => {
                write!(
                    f,
                    "Icon specified does not match binary image data nor UTF-8 encoded .svg data."
                )
            }
            IconValidationError::NotSquare => {
                write!(f, "Icon is not square!")
            }
        }
    }
}

impl From<std::io::Error> for EntryManagerError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<ValidationError> for EntryManagerError {
    fn from(value: ValidationError) -> Self {
        Self::EntryValidation(value)
    }
}

impl From<ImageError> for EntryManagerError {
    fn from(value: ImageError) -> Self {
        Self::IconValidation(IconValidationError::ImageFormat(value))
    }
}

impl From<ron::Error> for EntryManagerError {
    fn from(value: ron::Error) -> Self {
        Self::Ron(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Lifetime {
    Process(u32),
    Session(String),
    Persistent(String),
}

impl Lifetime {
    pub fn from_pid(pid: u32) -> Result<Self, ()> {
        Ok(Lifetime::Process(pid))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryManager {
    pub cache: EntryCache,
    pub temp_entry_dir: PathBuf,
    pub temp_icon_dir: PathBuf,
    pub persistent_entry_dir: PathBuf,
    pub persistent_icon_dir: PathBuf,
    pub config_file: PathBuf,
    pub change_handlers: HashSet<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryCache {
    pub entries: HashMap<Lifetime, Vec<DesktopHandle>>,
    pub icons: HashMap<Lifetime, Vec<IconHandle>>,
}

pub enum ConfigError {
    IO(std::io::Error),
    Parse(SpannedError),
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<SpannedError> for ConfigError {
    fn from(value: SpannedError) -> Self {
        Self::Parse(value)
    }
}

impl EntryCache {
    pub fn new(config_dir: &Path) -> Result<Self, ConfigError> {
        let str_data = fs::read_to_string(config_dir)?;
        let cache: Self = ron::from_str(&str_data)?;
        Ok(cache)
    }
}

impl EntryManager {
    pub fn new(tmp_dir: PathBuf, persistent_dir: PathBuf, config_file: PathBuf) -> Self {
        let mut manager = Self {
            cache: EntryCache::new(&config_file).unwrap_or_default(),
            temp_entry_dir: tmp_dir.join(Path::new("applications")),
            temp_icon_dir: tmp_dir.join(Path::new("icons")),
            persistent_entry_dir: persistent_dir.join(Path::new("applications")),
            persistent_icon_dir: persistent_dir.join(Path::new("icons")),
            config_file,
            change_handlers: HashSet::new(),
        };
        if let Err(e) = manager.reset_session() {
            log::warn!(
                "there was a problem resetting the session lifetime: {:?}",
                e
            );
        }
        manager
    }
    /// responsible for registering a desktop `entry` with a given `lifetime`. saves file as
    /// `appid`.desktop, and can be referred to with the specified appid
    pub fn register_entry(
        &mut self,
        entry: &str,
        appid: &str,
        lifetime: Lifetime,
    ) -> Result<(), EntryManagerError> {
        // validate entry
        let entry = validate_desktop_entry(entry, appid)?;
        let desktop_file_path = self
            .temp_entry_dir
            .as_path()
            .join(format!("{}.desktop", appid));
        if desktop_file_path.exists() {
            return Err(EntryManagerError::PathCollision(desktop_file_path));
        }
        // create applications directory just in case
        let _ = create_dir_all(desktop_file_path.parent().unwrap());
        std::fs::write(&desktop_file_path, entry.as_bytes())?;
        // add entry to the list of entries recorded
        match self.cache.entries.get_mut(&lifetime) {
            None => {
                self.cache
                    .entries
                    .insert(lifetime, vec![DesktopHandle::from(desktop_file_path)]);
            }
            Some(e) => {
                e.push(DesktopHandle::from(desktop_file_path));
            }
        }
        // resave cache
        self.save_cache()?;
        log::info!("Successfully entered: {} into the registry.", appid);
        Ok(())
    }

    /// responsible for registering an application icon with the given `icon_name`
    /// icon will have the specified `lifetime`
    pub fn register_icon(
        &mut self,
        icon_name: &str,
        icon_data: &[u8],
        lifetime: Lifetime,
    ) -> Result<(), EntryManagerError> {
        let icon_path = if let Ok(img) = image::io::Reader::new(std::io::Cursor::new(icon_data))
            .with_guessed_format()
            .unwrap()
            .decode()
        {
            // image sent as bytes
            self.icon_as_bytes(&img, icon_name)?
        } else if let Ok(text_data) = String::from_utf8(icon_data.into()) {
            // image sent as svg
            self.icon_as_svg(text_data, icon_name)?
        } else {
            return Err(EntryManagerError::IconValidation(
                IconValidationError::NoTypeFound,
            ));
        };
        // add entry to the list of entries recorded
        match self.cache.icons.get_mut(&lifetime) {
            None => {
                self.cache
                    .icons
                    .insert(lifetime, vec![IconHandle::from(icon_path)]);
            }
            Some(e) => {
                e.push(IconHandle::from(icon_path));
            }
        }
        // resave cache
        self.save_cache()?;
        Ok(())
    }

    fn icon_as_bytes(
        &self,
        img: &DynamicImage,
        icon_name: &str,
    ) -> Result<PathBuf, EntryManagerError> {
        log::info!("{} is a valid image as bytes", icon_name);
        if img.width() != img.height() {
            return Err(EntryManagerError::IconValidation(
                IconValidationError::NotSquare,
            ));
        }
        // only soft warn if the size is > 512
        let img = if img.width() > 512 {
            log::warn!("Image size was greater than 512! Resizing icon to 512x512.");
            img.resize(512, 512, image::imageops::FilterType::Lanczos3)
        } else {
            img.to_owned()
        };
        let icon_path = self.temp_icon_dir.join(Path::new(&format!(
            "hicolor/{}x{}/apps/{}.png",
            img.width(),
            img.height(),
            icon_name
        )));
        let _ = create_dir_all(icon_path.parent().unwrap());
        if icon_path.exists() {
            return Err(EntryManagerError::PathCollision(icon_path));
        }
        img.save_with_format(&icon_path, image::ImageFormat::Png)?;
        Ok(icon_path)
    }

    fn icon_as_svg(&self, svg_text: String, icon_name: &str) -> Result<PathBuf, EntryManagerError> {
        // check for valid svg
        svg::read(&svg_text)?;
        let icon_path = self.temp_icon_dir.join(Path::new(&format!(
            "hicolor/scalable/apps/{}.svg",
            icon_name
        )));
        let _ = create_dir_all(icon_path.parent().unwrap());
        if icon_path.exists() {
            return Err(EntryManagerError::PathCollision(icon_path));
        }
        fs::write(&icon_path, svg_text.as_bytes())?;
        Ok(icon_path)
    }

    pub fn remove_lifetime(&mut self, lifetime: Lifetime) -> Result<(), EntryManagerError> {
        log::info!("Deleting lifetime {:?}", lifetime);
        let mut changed = false;
        if let Some(entries) = self.cache.entries.get(&lifetime) {
            for entry in entries {
                match entry.clone().delete_self() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("problem deleting entry {:?} : {:?}", entry.appid, e);
                    }
                }
            }
            self.cache.entries.remove(&lifetime);
            changed = true;
        }
        if let Some(icons) = self.cache.icons.get(&lifetime) {
            for icon in icons {
                match icon.clone().delete_self() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("problem deleting icon {:?} : {:?}", icon.icon_name, e);
                    }
                }
            }
            self.cache.icons.remove(&lifetime);
            changed = true;
        }
        if changed {
            self.save_cache()?;
        }
        Ok(())
    }

    pub fn reset_session(&mut self) -> Result<(), EntryManagerError> {
        for lifetime in self
            .cache
            .entries
            .clone()
            .iter()
            .map(|x| x.0)
            .chain(self.cache.icons.clone().iter().map(|x| x.0))
            .filter_map(|x| {
                if matches!(x, Lifetime::Session(_)) {
                    Some(x.clone())
                } else {
                    None
                }
            })
        {
            self.remove_lifetime(lifetime.clone())?;
        }
        Ok(())
    }

    fn save_cache(&self) -> Result<(), EntryManagerError> {
        let conf_str = ron::ser::to_string_pretty(&self.cache, ron::ser::PrettyConfig::default())?;
        fs::write(&self.config_file, conf_str)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopHandle {
    pub appid: String,
    pub path: PathBuf,
}

impl Hash for DesktopHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.appid.hash(state);
    }
}

impl From<PathBuf> for DesktopHandle {
    fn from(value: PathBuf) -> Self {
        Self {
            appid: value.file_stem().unwrap().to_str().unwrap().to_string(),
            path: value,
        }
    }
}

impl DesktopHandle {
    fn delete_self(self) -> Result<(), std::io::Error> {
        fs::remove_file(&self.path)?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IconHandle {
    pub icon_name: String,
    pub icon_path: PathBuf,
}

impl Hash for IconHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.icon_name.hash(state);
    }
}

impl From<PathBuf> for IconHandle {
    fn from(value: PathBuf) -> Self {
        Self {
            icon_name: value.file_stem().unwrap().to_str().unwrap().to_string(),
            icon_path: value,
        }
    }
}

impl IconHandle {
    fn delete_self(self) -> Result<(), std::io::Error> {
        fs::remove_file(&self.icon_path)?;
        Ok(())
    }
}
