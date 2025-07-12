use std::error::Error;
use std::path::PathBuf;

use crate::config::{Config, StoreConfig};
use crate::project;
use crate::{dirs, store};

pub struct TrackResult {
    pub id: project::ProjectID,
    pub store_results: Vec<(StoreConfig, StoreTrackResult)>,
}

type StoreTrackResult = Result<&'static str, Box<dyn Error>>;

pub fn track(
    cfg: Config,
    prog: project::Program,
    file_name: PathBuf,
) -> Result<TrackResult, Box<dyn Error>> {
    let id = project::ProjectID {
        program: prog,
        name: file_name.to_string_lossy().to_string(),
    };

    let dirs = dirs::Dirs::new(&cfg)?;
    let archive = match project::read(&id, &dirs)? {
        Some(archive) => archive,
        None => return Err(format!("Project file not found: {}", file_name.display()).into()),
    };

    let update = vec![(&id, &archive)];

    let commit_message = format!("Start tracking {id}");

    let mut store_results = Vec::new();
    let (stores, store_errs) = store::open_all(&cfg.stores);
    for (st, err) in store_errs {
        store_results.push((st, Err(err)));
    }
    for (st, store) in stores {
        store_results.push((st, store.commit(&update, &commit_message)));
    }
    Ok(TrackResult { id, store_results })
}
