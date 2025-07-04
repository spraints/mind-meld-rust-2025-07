use std::path::PathBuf;

use crate::config::Config;

pub struct Dirs {
    pub mindstorms: PathBuf,
    pub spike: PathBuf,
}

impl Dirs {
    pub fn new(config: &Config) -> Result<Self, &'static str> {
        let mindstorms = match &config.mindstorms_path {
            None => default_mindstorms()?,
            Some(p) => p.into(),
        };
        let spike = match &config.spike_path {
            None => default_spike()?,
            Some(p) => p.into(),
        };
        Ok(Self { mindstorms, spike })
    }
}

#[cfg(target_os = "macos")]
fn default_spike() -> Result<PathBuf, &'static str> {
    let home_dir = std::env::home_dir().ok_or("could not determine home directory")?;
    Ok(home_dir.join(
        "Library/Containers/com.lego.education.spikenext/Data/Documents/LEGO Education SPIKE",
    ))
}

#[cfg(target_os = "macos")]
fn default_mindstorms() -> Result<PathBuf, &'static str> {
    let home_dir = std::env::home_dir().ok_or("could not determine home directory")?;
    Ok( home_dir.join("Library/Containers/com.lego.retail.mindstorms.robotinventor/Data/Documents/LEGO MINDSTORMS"))
}

#[cfg(target_os = "windows")]
fn default_spike() -> Result<PathBuf, &'static str> {
    let home_dir = std::env::home_dir().ok_or("could not determine home directory")?;
    Ok(home_dir.join("Documents/LEGO MINDSTORMS"))
}

#[cfg(target_os = "windows")]
fn default_mindstorms() -> Result<PathBuf, &'static str> {
    let home_dir = std::env::home_dir().ok_or("could not determine home directory")?;
    Ok(home_dir.join("Documents/LEGO MINDSTORMS"))
}
