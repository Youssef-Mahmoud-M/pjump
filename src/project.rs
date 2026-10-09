use std::fmt::Display;

mod fs_handlers;

pub use fs_handlers::*;

pub struct Project {
    name: String,
    path: String,
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.name, self.path)
    }
}

impl Project {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn path(&self) -> &String {
        &self.path
    }
}
