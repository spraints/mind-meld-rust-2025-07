pub mod fs;
pub mod store;
pub mod txt;

use std::error::Error;

use crate::project::types::ProjectType;
use crate::project::{Project, ProjectID, RawProject};
use crate::store::{Revision, Store};

pub(crate) fn render_all_projects(
    mut dest: impl RenderDest,
    fmt: impl ProjectFormatter,
    store: &Store,
    revision: Revision,
) -> Result<(), Box<dyn Error>> {
    dest.pre_flight(&revision)?;
    for proj_id in store.project_ids()? {
        match store.read_project(&proj_id, &revision)? {
            None => println!("{proj_id}! missing, oddly."),
            Some(p) => match render_project(&mut dest, &fmt, &proj_id, p) {
                Ok(Some(msg)) => println!("{proj_id}: {msg}"),
                Ok(None) => println!("{proj_id}: rendered"),
                Err(e) => println!("{proj_id}! {e}"),
            },
        };
    }
    match dest.finish() {
        Ok(Some(msg)) => {
            println!("{msg}");
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(e) => Err(e),
    }
}

pub trait RenderDest {
    fn pre_flight(&mut self, revision: &Revision) -> Result<(), Box<dyn Error>>;

    fn write(
        &mut self,
        proj_id: &ProjectID,
        proj_type: ProjectType,
        content: &[u8],
    ) -> Result<Option<String>, Box<dyn Error>>;

    fn finish(self) -> Result<Option<String>, Box<dyn Error>>;
}

pub trait ProjectFormatter {
    fn render(&self, proj: &Project) -> Result<Vec<u8>, Box<dyn Error>>;
}

fn render_project(
    dest: &mut impl RenderDest,
    fmt: &impl ProjectFormatter,
    proj_id: &ProjectID,
    proj_content: RawProject,
) -> Result<Option<String>, Box<dyn Error>> {
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
