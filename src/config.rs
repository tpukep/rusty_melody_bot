use std::fs;
use std::path::Path;
use std::io::prelude::*;
use super::toml;
use super::errors::{Result, ResultExt, ErrorKind};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub telegram: Telegram,
    pub database:  Database
}

#[derive(Debug, Clone, Deserialize)]
pub struct Telegram {
    pub token:    String
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    pub path:    String
}

pub fn get<P: AsRef<Path>>(path: P) -> Result<AppConfig> {
    let path = path.as_ref();
    let mut config_file = fs::File::open(path).chain_err(|| {
        ErrorKind::Config(path.to_string_lossy().into_owned(), "Cannot open")
    })?;

    let mut content = String::new();
    config_file.read_to_string(&mut content).chain_err(|| {
        ErrorKind::Config(path.to_string_lossy().into_owned(), "Error in reading file")
    })?;

    let app_confg: AppConfig = toml::from_str(&content).chain_err(|| {
                ErrorKind::Config(path.to_string_lossy().into_owned(), "Error in conf file")
            })?;

    Ok(app_confg)
}
