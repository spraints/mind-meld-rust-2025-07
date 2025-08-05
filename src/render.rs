pub mod fs;

use std::error::Error;

use crate::project::{ArchiveEntryContents, ProjectID, RawArchive, RawProject};

pub(crate) fn render_all_projects(
    dest: impl RenderDest,
    store: crate::store::Store,
    revision: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let rev = revision.as_ref().map(|x| x.as_str());
    for proj_id in store.project_ids()? {
        match store.read_project(&proj_id, rev)? {
            None => println!("{proj_id}: missing, oddly."),
            Some(p) => render_project(&dest, &proj_id, &p)?,
        };
    }
    Ok(())
}

pub trait RenderDest {}

fn render_project(
    _: &impl RenderDest,
    proj_id: &ProjectID,
    proj_content: &RawProject,
) -> Result<(), Box<dyn Error>> {
    dbg_archive(proj_id, "", &proj_content.archive);
    Ok(())
}

fn dbg_archive(proj_id: &ProjectID, prefix: &str, arch: &RawArchive) {
    for e in &arch.entries {
        match &e.contents {
            ArchiveEntryContents::Data(_) => println!("!! TODO !! {proj_id} !! {prefix}{}", e.name),
            ArchiveEntryContents::Archive(child) => {
                let child_prefix = format!("{prefix}{}/", e.name);
                dbg_archive(proj_id, &child_prefix, &child);
            }
        };
    }
}
