use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub stores: Vec<StoreConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreConfig {
    pub path: String,
    #[serde(rename = "type")]
    pub store_type: String,
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut dir| {
        dir.push("mind-meld.toml");
        dir
    })
}

impl Config {
    pub fn load() -> io::Result<Self> {
        let path = match config_path() {
            Some(p) => p,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "No config dir found")),
        };
        if !path.exists() {
            return Ok(Config::default());
        }
        let contents = fs::read_to_string(&path)?;
        let config = toml::from_str(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    pub fn store(&self) -> io::Result<()> {
        let path = match config_path() {
            Some(p) => p,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "No config dir found")),
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mut file = fs::File::create(&path)?;
        file.write_all(toml.as_bytes())?;
        Ok(())
    }
} 