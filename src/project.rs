use crate::program::Program;

#[derive(Eq, Hash, PartialEq)]
pub struct ProjectID {
    program: Program,
    name: String,
}
