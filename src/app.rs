use std::error::Error;
use std::fs::read_dir;
use std::path::PathBuf;

use crate::dirs::Dirs;
use crate::project::*;

pub fn all_projects(dirs: Dirs) -> Result<Vec<ProjectID>, Box<dyn Error>> {
    let mut res = Vec::new();
    for (prog, path) in all_programs(dirs) {
        let mut pp = projects(prog, path)?;
        res.append(&mut pp);
    }
    Ok(res)
}

fn projects(prog: Program, path: PathBuf) -> Result<Vec<ProjectID>, Box<dyn Error>> {
    let mut res = Vec::new();
    for entry in read_dir(&path)? {
        let entry = entry?;
        if entry.metadata()?.is_file() {
            res.push(ProjectID {
                program: prog,
                name: entry.file_name().to_string_lossy().to_string(),
            });
        }
    }
    Ok(res)
}
