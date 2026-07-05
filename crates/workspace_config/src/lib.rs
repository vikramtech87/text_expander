use std::{fs, io};
use std::path::PathBuf;

pub const APP_DIR_NAME: &str = "RTextExpander";
pub const RULES_FILE_NAME: &str = "rules.toml";

pub fn get_config_dir() -> Result<PathBuf, io::Error> {
    let mut path = dirs_next::config_dir()
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Could not resolve system config directory")
        })?;

    path.push(APP_DIR_NAME);

    fs::create_dir_all(&path)?;

    Ok(path)
}

pub fn get_rules_file() -> Result<PathBuf, io::Error> {
    let mut path = get_config_dir()?;
    path.push(RULES_FILE_NAME);
    Ok(path)
}