use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use crate::project::types::ProjectType;
use crate::store::Revision;

pub(crate) fn out_dir(path: PathBuf) -> OutDir {
    OutDir { path }
}

pub struct OutDir {
    path: PathBuf,
}

impl super::RenderDest for OutDir {
    fn pre_flight(&mut self, _: &Revision) -> Result<(), Box<dyn std::error::Error>> {
        match fs::create_dir(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if matches!(e.kind(), ErrorKind::AlreadyExists) => Err(format!(
                "{:?}: directory already exists, remove it to render there again.",
                self.path
            )
            .into()),
            Err(e) => Err(e.into()),
        }
    }

    fn write(
        &mut self,
        proj_id: &crate::project::ProjectID,
        proj_type: ProjectType,
        content: &[u8],
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let out_dir = self.path.join(format!("{}", proj_id.program));
        fs::create_dir_all(&out_dir)?;

        let out_path = out_dir
            .join(&proj_id.name)
            .with_extension(proj_type.extension());
        fs::write(&out_path, content)?;
        Ok(Some(format!("rendered to {out_path:?}")))
    }

    fn finish(self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(None)
    }
}
