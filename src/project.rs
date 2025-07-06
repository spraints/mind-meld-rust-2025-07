use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::PathBuf;

use zip::ZipArchive;

use crate::dirs::Dirs;

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum Program {
    Mindstorms,
    Spike,
}

pub fn all_programs(dirs: &Dirs) -> Vec<(Program, &PathBuf)> {
    vec![
        (Program::Mindstorms, &dirs.mindstorms),
        (Program::Spike, &dirs.spike),
    ]
}

pub fn read(id: &ProjectID, dirs: &Dirs) -> Result<RawProject, Box<dyn Error>> {
    let base_path = dir(id.program, dirs);
    let path = base_path.join(&id.name);
    let archive = ZipArchive::new(File::open(path)?)?;
    Ok(RawProject {
        archive: RawArchive::read(archive)?,
    })
}

fn dir(prog: Program, dirs: &Dirs) -> &PathBuf {
    match prog {
        Program::Mindstorms => &dirs.mindstorms,
        Program::Spike => &dirs.spike,
    }
}

const PROGRAM_MINDSTORMS: &str = "mindstorms";
const PROGRAM_SPIKE: &str = "spike";

pub fn program_git(name: &gix::bstr::BStr) -> Result<Program, String> {
    if name == PROGRAM_MINDSTORMS {
        Ok(Program::Mindstorms)
    } else if name == PROGRAM_SPIKE {
        Ok(Program::Spike)
    } else {
        Err(format!("invalid program {name:?}"))
    }
}

#[derive(Eq, Hash, PartialEq)]
pub struct ProjectID {
    pub(crate) program: Program,
    pub(crate) name: String,
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Program::Mindstorms => write!(f, "{PROGRAM_MINDSTORMS}"),
            Program::Spike => write!(f, "{PROGRAM_SPIKE}"),
        }
    }
}

impl Display for ProjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.program, self.name)
    }
}

/// RawProject has the contents of a lms or llsp3 file.
pub struct RawProject {
    /// The entries within the zip file.
    pub archive: RawArchive,
}

pub struct RawArchive {
    pub entries: Vec<ArchiveEntry>,
}
impl RawArchive {
    fn read<R: Read + Seek>(mut archive: ZipArchive<R>) -> Result<Self, Box<dyn Error>> {
        let mut entries = Vec::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            entries.push(ArchiveEntry::new(file.name(), buf)?);
        }
        Ok(Self { entries })
    }
}

pub struct ArchiveEntry {
    pub name: String,
    pub contents: ArchiveEntryContents,
}

impl ArchiveEntry {
    fn new(name: &str, buf: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let contents = match name {
            "scratch.sb3" => {
                let archive = ZipArchive::new(std::io::Cursor::new(buf))?;
                ArchiveEntryContents::Archive(RawArchive::read(archive)?)
            }
            _ => ArchiveEntryContents::Data(buf),
        };
        let name = name.to_string();
        Ok(Self { name, contents })
    }
}

pub enum ArchiveEntryContents {
    Data(Vec<u8>),
    Archive(RawArchive),
}
