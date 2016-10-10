use ci::{self, job};

use std::path::PathBuf;

/// A ci.
pub struct Project
{
    /// The project name.
    pub name: String,
    /// The file cache.
    pub cache: ci::Cache,
    /// The sandboxing implementation.
    pub sandbox: Box<ci::Sandbox>,
}

impl Project
{
    pub fn new(name: String, directory: PathBuf) -> Self {
        Project {
            name: name,
            cache: ci::Cache::new(directory),
            sandbox: Box::new(ci::sandbox::Basic),
        }
    }

    pub fn run(&mut self, command: job::Command)
        -> ci::build::TaskOutput {
        self.sandbox.run(command, self.cache.directory())
    }
}


