use std::error::Error;

use crate::config::StoreConfig;
use crate::dirs::Dirs;
use crate::project::{self, ProjectID};
use crate::store::{self, Store};

pub struct CommitResult {
    pub missing_projects: Vec<ProjectID>,
    pub project_read_errors: Vec<(ProjectID, Box<dyn Error>)>,
    pub store_results: Vec<(StoreConfig, store::CommitResult)>,
}

pub fn commit<'a, P: IntoIterator<Item = &'a ProjectID>>(
    stores: &[(StoreConfig, Store)],
    dirs: &Dirs,
    projects: P,
    message: &str,
) -> CommitResult {
    let mut missing_projects = Vec::new();
    let mut project_read_errors = Vec::new();
    let mut projects_to_commit = Vec::new();
    for proj_id in projects {
        match project::read(proj_id, dirs) {
            Ok(Some(raw_project)) => projects_to_commit.push((proj_id.clone(), raw_project)),
            Ok(None) => missing_projects.push(proj_id.clone()),
            Err(e) => project_read_errors.push((proj_id.clone(), e)),
        };
    }

    let mut store_results = Vec::new();
    for (st, store) in stores {
        store_results.push((st.clone(), store.commit(&projects_to_commit, message)));
    }

    CommitResult {
        missing_projects,
        project_read_errors,
        store_results,
    }
}
