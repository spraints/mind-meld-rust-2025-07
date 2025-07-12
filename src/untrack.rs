use std::error::Error;
use std::path::PathBuf;

use crate::config::{Config, StoreConfig};
use crate::project;
use crate::store;

pub struct UntrackResult {
    pub id: project::ProjectID,
    pub store_results: Vec<(StoreConfig, StoreUntrackResult)>,
}

type StoreUntrackResult = Result<&'static str, Box<dyn Error>>;

pub fn untrack(
    cfg: Config,
    prog: project::Program,
    file_name: PathBuf,
) -> Result<UntrackResult, Box<dyn Error>> {
    let id = project::ProjectID {
        program: prog,
        name: file_name.to_string_lossy().to_string(),
    };

    let mut store_results = Vec::new();
    let (stores, store_errs) = store::open_all(&cfg.stores);
    for (st, err) in store_errs {
        store_results.push((st, Err(err)));
    }
    for (st, store) in stores {
        store_results.push((st, store.untrack(&id, &format!("Stop tracking {id}"))));
    }
    Ok(UntrackResult { id, store_results })
}
