use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub stores: Vec<StoreConfig>,

    pub mindstorms_path: Option<PathBuf>,
    pub spike_path: Option<PathBuf>,

    #[serde(skip)]
    config_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreConfig {
    pub path: PathBuf,
    #[serde(rename = "type")]
    pub store_type: String,
}

impl Display for StoreConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} ({})", self.path, self.store_type)
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut dir| {
        dir.push("mind-meld.toml");
        dir
    })
}

fn get_config_path<P: Into<PathBuf>>(path: Option<P>) -> io::Result<PathBuf> {
    match path {
        Some(p) => Ok(p.into()),
        None => config_path()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No config dir found")),
    }
}

impl Config {
    pub fn load(path: Option<&str>) -> io::Result<Self> {
        let path = get_config_path(path)?;
        if !path.exists() {
            return Ok(Config::default());
        }
        let contents = fs::read_to_string(&path)?;
        let mut cfg = Self::load_from_string(&contents)?;
        cfg.config_path = Some(path);
        Ok(cfg)
    }

    fn load_from_string(contents: &str) -> io::Result<Self> {
        let config =
            toml::from_str(contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    pub fn store(&self) -> io::Result<()> {
        let path = get_config_path(self.config_path.as_deref())?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string_parses() {
        let toml = "";
        let config: Config = Config::load_from_string(toml).unwrap();
        assert_eq!(config.stores.len(), 0);
        assert_eq!(config.mindstorms_path, None);
        assert_eq!(config.spike_path, None);
    }

    #[test]
    fn test_extra_info_parses() {
        let toml = r#"
        foo = "bar"
        [[stores]]
        path = "p"
        type = "git"
        extra = 123
        "#;
        let config: Config = Config::load_from_string(toml).unwrap();
        assert_eq!(config.stores.len(), 1);
        assert_eq!(config.stores[0].path, "p");
        assert_eq!(config.stores[0].store_type, "git");
    }

    #[test]
    fn test_example_parses() {
        let toml = r#"
        [[stores]]
        path = "path1"
        type = "git"
        [[stores]]
        path = "path2"
        type = "git"
        "#;
        let config: Config = Config::load_from_string(toml).unwrap();
        assert_eq!(config.stores.len(), 2);
        assert_eq!(config.stores[0].path, "path1");
        assert_eq!(config.stores[0].store_type, "git");
        assert_eq!(config.stores[1].path, "path2");
        assert_eq!(config.stores[1].store_type, "git");
    }

    #[test]
    fn test_overrides_parses() {
        let toml = r#"
        mindstorms_path = "path/to/mindstorms"
        spike_path = "path/to/spike"
        "#;
        let config: Config = Config::load_from_string(toml).unwrap();
        assert_eq!(
            config.mindstorms_path,
            Some("path/to/mindstorms".to_string())
        );
        assert_eq!(config.spike_path, Some("path/to/spike".to_string()));
    }
}
