use std::error::Error;
use std::rc::Rc;

use crate::{
    dirs::Dirs,
    project::{self, ProjectID},
    store::Store,
};

pub enum Status {
    NoDifferences,
    LocalMissing,
    Differences(Vec<Rc<Store>>),
}

pub fn get_status(
    proj: &ProjectID,
    stores: &[Rc<Store>],
    dirs: &Dirs,
) -> Result<Status, Box<dyn Error>> {
    let local = match project::read(proj, dirs) {
        Ok(Some(project)) => project,
        Ok(None) => return Ok(Status::LocalMissing),
        Err(e) => {
            if let Some(io_error) = e.downcast_ref::<std::io::Error>() {
                if io_error.kind() == std::io::ErrorKind::NotFound {
                    return Ok(Status::LocalMissing);
                }
            }
            return Err(e);
        }
    };
    let local_hash = local.hash();

    let mut diff = Vec::new();
    for st in stores {
        match st.read_project(proj) {
            Err(_) => diff.push(st.clone()),
            Ok(None) => diff.push(st.clone()),
            Ok(Some(st_proj)) => {
                let st_hash = st_proj.hash();
                if st_hash != local_hash {
                    diff.push(st.clone());
                }
            }
        };
    }

    match diff.is_empty() {
        true => Ok(Status::NoDifferences),
        false => Ok(Status::Differences(diff)),
    }
}
