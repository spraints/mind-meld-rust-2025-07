use std::path::PathBuf;

pub(crate) fn out_dir(path: PathBuf) -> OutDir {
    OutDir { path }
}

pub struct OutDir {
    path: PathBuf,
}

impl super::RenderDest for OutDir {}
