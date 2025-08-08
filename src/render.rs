pub mod fs;
pub mod txt;

use std::error::Error;
use std::path::PathBuf;

use crate::project::{
    types::ProjectType, ArchiveEntryContents, Project, ProjectID, RawArchive, RawProject,
};

pub(crate) fn render_all_projects(
    dest: impl RenderDest,
    fmt: impl ProjectFormatter,
    store: crate::store::Store,
    revision: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let rev = revision.as_ref().map(|x| x.as_str());
    for proj_id in store.project_ids()? {
        match store.read_project(&proj_id, rev)? {
            None => println!("{proj_id}! missing, oddly."),
            Some(p) => match render_project(&dest, &fmt, &proj_id, p) {
                Ok(path) => println!("{proj_id}: ok => {path:?}"),
                Err(e) => println!("{proj_id}! {e}"),
            },
        };
    }
    Ok(())
}

pub trait RenderDest {
    fn write(
        &self,
        proj_id: &ProjectID,
        proj_type: ProjectType,
        content: &[u8],
    ) -> Result<PathBuf, Box<dyn Error>>;
}

pub trait ProjectFormatter {
    fn render(&self, proj: &Project) -> Result<Vec<u8>, Box<dyn Error>>;
}

fn render_project(
    dest: &impl RenderDest,
    fmt: &impl ProjectFormatter,
    proj_id: &ProjectID,
    proj_content: RawProject,
) -> Result<PathBuf, Box<dyn Error>> {
    let proj = proj_content.into_project()?;
    let rendered = fmt.render(&proj)?;
    dest.write(proj_id, proj.project_type(), &rendered)
}

// fn dbg_archive(proj_id: &ProjectID, prefix: &str, arch: &RawArchive) {
//     for e in &arch.entries {
//         match &e.contents {
//             ArchiveEntryContents::Data(_) => println!("!! TODO !! {proj_id} !! {prefix}{}", e.name),
//             ArchiveEntryContents::Archive(child) => {
//                 let child_prefix = format!("{prefix}{}/", e.name);
//                 dbg_archive(proj_id, &child_prefix, &child);
//             }
//         };
//     }
// }
