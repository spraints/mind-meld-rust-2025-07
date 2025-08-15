use crate::store::{Revision, Store};

use super::RenderDest;

pub fn tree(store: &Store) -> StoreRenderer {
    StoreRenderer {
        store,
        revision: Revision::Latest,
        rendered: Default::default(),
    }
}

pub struct StoreRenderer<'a> {
    store: &'a Store,
    revision: Revision,
    rendered: Vec<(String, Vec<u8>)>,
}

impl<'a> RenderDest for StoreRenderer<'a> {
    fn pre_flight(&mut self, revision: &Revision) -> Result<(), Box<dyn std::error::Error>> {
        self.revision = revision.clone();
        Ok(())
    }

    fn write(
        &mut self,
        proj_id: &crate::project::ProjectID,
        proj_type: crate::project::types::ProjectType,
        content: &[u8],
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let path = format!(
            "{}/{}.{}",
            proj_id.program,
            drop_extension(&proj_id.name),
            proj_type.extension()
        );
        self.rendered.push((path, content.to_vec()));
        Ok(None)
    }

    fn finish(self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let msg = format!("Rendered {}", self.revision);
        let commit_id = self.store.store_render(
            &self.rendered,
            &msg,
            Revision::Empty,
            self.revision.clone(),
        )?;
        Ok(Some(format!("created commit {commit_id}")))
    }
}

fn drop_extension(path: &str) -> &str {
    match path.rsplit_once(".") {
        Some((prefix, _)) => prefix,
        None => path,
    }
}
