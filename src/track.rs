use std::error::Error;
use std::path::PathBuf;

use crate::config::{Config, StoreConfig};
use crate::project;
use crate::{dirs, store};

pub struct TrackResult {
    id: project::ProjectID,
    store_results: Vec<(StoreConfig, Result<(), Box<dyn Error>>)>,
}

pub fn track(
    cfg: Config,
    prog: project::Program,
    file_name: PathBuf,
) -> Result<TrackResult, Box<dyn Error>> {
    let id = project::ProjectID {
        program: prog,
        name: file_name.to_string_lossy().to_string(),
    };

    let archive = project::read(&id, dirs::Dirs::new(&cfg)?)?;

    let update = vec![(&id, &archive)];

    let mut store_results = Vec::new();
    for st in cfg.stores {
        let res = match store::open(&st) {
            Err(e) => Err(e),
            Ok(store) => store.commit(&update),
        };
        store_results.push((st, res));
    }
    Ok(TrackResult { id, store_results })
}
