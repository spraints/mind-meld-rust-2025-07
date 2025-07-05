use std::error::Error;
use std::path::Path;

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
        match (e.mode(), e.filename()) {
            (m, _) if m.is_tree() => {}
            (_, f) if f == "mindstorms" => {}
            (_, f) if f == "spike" => {}
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
