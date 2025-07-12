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
    for st in cfg.stores {
        let res = match store::open(&st) {
            Err(e) => Err(e),
            Ok(store) => store.untrack(&id, &format!("Stop tracking {id}")),
        };
        store_results.push((st, res));
    }
    Ok(UntrackResult { id, store_results })
}
