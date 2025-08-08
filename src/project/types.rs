use serde::Deserialize;

#[derive(Deserialize)]
pub struct Manifest {
    #[serde(rename = "type")]
    pub project_type: ProjectType,
}

#[derive(Deserialize)]
pub enum ProjectType {
    #[serde(rename = "word-blocks")]
    WordBlocks,
    #[serde(rename = "icon-blocks")]
    IconBlocks,
    #[serde(rename = "python")]
    Python,
}
