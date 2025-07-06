use std::error::Error;
use std::path::Path;

use crate::project::{program_git, ProjectID};

pub fn open<P: AsRef<Path>>(p: P) -> Result<GitStore, Box<dyn Error>> {
    let r = gix::discover(&p)?;
    GitStore::new(r)
}

pub fn create<P: AsRef<Path>>(p: P) -> Result<GitStore, Box<dyn Error>> {
    let r = gix::init_bare(p)?;
    GitStore::new(r)
}

pub struct GitStore {
    r: gix::Repository,
}

impl GitStore {
    fn new(r: gix::Repository) -> Result<Self, Box<dyn Error>> {
        validate(&r)?;
        Ok(Self { r })
    }

    pub fn project_ids(&self) -> Result<Vec<crate::project::ProjectID>, Box<dyn Error + 'static>> {
        if self.r.head()?.is_unborn() {
            return Ok(Vec::new());
        }
        let mut res = Vec::new();
        for e in self.r.head_commit()?.tree()?.iter() {
            let e = e?;
            match (e.mode().is_tree(), program_git(e.filename())) {
                (false, _) => (), // skip
                (true, p) => {
                    let p = p?;
                    for proj in e.object()?.try_into_tree()?.iter() {
                        res.push(ProjectID {
                            program: p,
                            name: proj?.filename().to_string(),
                        });
                    }
                }
            };
        }
        Ok(res)
    }

    pub(crate) fn commit(
        &self,
        projects: Vec<(&ProjectID, &crate::project::RawProject)>,
    ) -> Result<(), Box<dyn Error + 'static>> {
        todo!()
    }
}

fn validate(r: &gix::Repository) -> Result<(), Box<dyn Error>> {
    if r.head()?.is_unborn() {
        return Ok(());
    }
    validate_tree(r.head_commit()?.tree()?)
}

fn validate_tree(t: gix::Tree) -> Result<(), Box<dyn Error>> {
    for e in t.iter() {
        let e = e?;
        if !e.mode().is_tree() {
            continue;
        }
        match (e.mode().is_tree(), program_git(e.filename())) {
            (true, Ok(_)) => {}
            (_, _) => {
                return Err(format!(
                    "repository can not be used for mind-meld, it has extra entries like {e}"
                )
                .into());
            }
        };
    }
    Ok(())
}
