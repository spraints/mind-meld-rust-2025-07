mod git;

use std::error::Error;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use crate::config::StoreConfig;
use crate::project::{self, ProjectID};

pub struct Store {
    path: PathBuf,
    inst: StoreInstance,
}

const STORE_TYPE_GIT: &str = "git";

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

pub fn open_all(
    scs: &[StoreConfig],
) -> (
    Vec<(StoreConfig, Store)>,
    Vec<(StoreConfig, Box<dyn Error>)>,
) {
    let mut ok = Vec::new();
    let mut errs = Vec::new();
    for st in scs {
        match open(st) {
            Ok(s) => ok.push((st.clone(), s)),
            Err(e) => errs.push((st.clone(), e)),
        };
    }
    (ok, errs)
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

    fn commit(
        &self,
        projects: &[(&ProjectID, &project::RawProject)],
        message: &str,
    ) -> Result<&'static str, Box<dyn Error>> {
        match self {
            Self::Git(s) => s.commit(projects, message),
        }
    }

    fn read_project(
        &self,
        id: &ProjectID,
    ) -> Result<Option<project::RawProject>, Box<dyn Error + 'static>> {
        match self {
            Self::Git(s) => s.read_project(id),
        }
    }

    fn untrack(&self, id: &ProjectID, message: &str) -> Result<&'static str, Box<dyn Error>> {
        match self {
            Self::Git(s) => s.untrack(id, message),
        }
    }
}

impl Store {
    pub fn project_ids(&self) -> Result<Vec<ProjectID>, Box<dyn Error>> {
        self.inst.project_ids()
    }

    pub fn read_project(
        &self,
        id: &ProjectID,
    ) -> Result<Option<project::RawProject>, Box<dyn Error>> {
        self.inst.read_project(id)
    }

    pub(crate) fn commit(
        &self,
        projects: &[(&ProjectID, &project::RawProject)],
        message: &str,
    ) -> Result<&'static str, Box<dyn Error>> {
        self.inst.commit(projects, message)
    }

    pub(crate) fn untrack(
        &self,
        id: &ProjectID,
        message: &str,
    ) -> Result<&'static str, Box<dyn Error>> {
        self.inst.untrack(id, message)
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

impl From<Store> for StoreConfig {
    fn from(val: Store) -> Self {
        StoreConfig {
            path: val.path,
            store_type: val.inst.store_type().as_str().to_string(),
        }
    }
}

pub fn paths_match<P1: AsRef<Path>, P2: AsRef<Path>>(p1: P1, p2: P2) -> bool {
    match (std::path::absolute(p1), std::path::absolute(p2)) {
        (Ok(p1), Ok(p2)) => p1 == p2,
        _ => false,
    }
}
