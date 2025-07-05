use std::fmt::Display;

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum Program {
    Mindstorms,
    Spike,
}

const PROGRAM_MINDSTORMS: &'static str = "mindstorms";
const PROGRAM_SPIKE: &'static str = "spike";

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
