mod git;

use std::error::Error;
use std::path::{Path, PathBuf};

use crate::config::StoreConfig;

pub struct Store {
    store_type: StoreType,
    path: PathBuf,
}

const STORE_TYPE_GIT: &'static str = "git";

enum StoreType {
    Git,
}

fn store_type(t: &str) -> Result<StoreType, String> {
    match t {
        STORE_TYPE_GIT => Ok(StoreType::Git),
        _ => Err(format!("invalid store type: {t}")),
    }
}

pub fn create(t: &str, path: PathBuf) -> Result<Store, Box<dyn Error>> {
    let path = std::path::absolute(path)?;
    let t = store_type(t)?;
    t.create(&path)?;
    Ok(Store {
        store_type: t,
        path: path,
    })
}

impl StoreType {
    fn create<P: AsRef<Path>>(&self, p: P) -> Result<(), Box<dyn Error>> {
        match self {
            StoreType::Git => git::open_or_create(p)?,
        };
        Ok(())
    }

    fn as_str(&self) -> &'static str {
        match self {
            StoreType::Git => STORE_TYPE_GIT,
        }
    }
}

impl Into<StoreConfig> for Store {
    fn into(self) -> StoreConfig {
        StoreConfig {
            path: self.path,
            store_type: self.store_type.as_str().to_string(),
        }
    }
}

pub fn paths_match<P1: AsRef<Path>, P2: AsRef<Path>>(p1: P1, p2: P2) -> bool {
    match (std::path::absolute(p1), std::path::absolute(p2)) {
        (Ok(p1), Ok(p2)) => p1 == p2,
        _ => false,
    }
}
