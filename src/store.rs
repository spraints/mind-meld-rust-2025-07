mod git;

use std::error::Error;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use crate::config::StoreConfig;
use crate::project::ProjectID;

pub struct Store {
    path: PathBuf,
    inst: StoreInstance,
}

const STORE_TYPE_GIT: &'static str = "git";

enum StoreType {
    Git,
}

enum StoreInstance {
    Git(git::GitStore),
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
    let inst = t.create(&path)?;
    Ok(Store { inst, path })
}

pub fn open(st: &StoreConfig) -> Result<Store, Box<dyn Error>> {
    let path = std::path::absolute(&st.path)?;
    let t = store_type(&st.store_type)?;
    let inst = t.open(&path)?;
    Ok(Store { inst, path })
}

impl StoreType {
    fn create<P: AsRef<Path>>(&self, p: P) -> Result<StoreInstance, Box<dyn Error>> {
        match self {
            StoreType::Git => Ok(StoreInstance::Git(
                git::open(&p).or_else(|_| git::create(&p))?,
            )),
        }
    }

    fn open<P: AsRef<Path>>(&self, p: P) -> Result<StoreInstance, Box<dyn Error>> {
        match self {
            StoreType::Git => Ok(StoreInstance::Git(git::open(p)?)),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            StoreType::Git => STORE_TYPE_GIT,
        }
    }
}

impl StoreInstance {
    fn store_type(&self) -> StoreType {
        match self {
            Self::Git(_) => StoreType::Git,
        }
    }

    fn project_ids(&self) -> Result<Vec<ProjectID>, Box<dyn Error + 'static>> {
        match self {
            Self::Git(s) => s.project_ids(),
        }
    }
}

impl Store {
    pub fn project_ids(&self) -> Result<Vec<ProjectID>, Box<dyn Error>> {
        self.inst.project_ids()
    }
}

impl Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = StoreConfig {
            path: self.path.clone(),
            store_type: self.inst.store_type().as_str().to_string(),
        };
        write!(f, "{c}")
    }
}

impl Into<StoreConfig> for Store {
    fn into(self) -> StoreConfig {
        StoreConfig {
            path: self.path,
            store_type: self.inst.store_type().as_str().to_string(),
        }
    }
}

pub fn paths_match<P1: AsRef<Path>, P2: AsRef<Path>>(p1: P1, p2: P2) -> bool {
    match (std::path::absolute(p1), std::path::absolute(p2)) {
        (Ok(p1), Ok(p2)) => p1 == p2,
        _ => false,
    }
}
