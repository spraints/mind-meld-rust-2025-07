pub struct TextFormatter;

use crate::project::{Project, PythonProject};

type RenderResult = Result<Vec<u8>, Box<dyn std::error::Error>>;

impl super::ProjectFormatter for TextFormatter {
    fn render(&self, proj: &Project) -> RenderResult {
        match proj {
            Project::Python(proj) => render_python(proj),
            Project::WordBlocks(_) => Err("todo: render word-blocks".into()),
            Project::IconBlocks(_) => Err("todo: render icon-blocks".into()),
        }
    }
}

fn render_python(proj: &PythonProject) -> RenderResult {
    Ok(proj.get_source()?.bytes().collect())
}
