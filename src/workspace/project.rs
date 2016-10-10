use workspace;
use job;

use std::path::PathBuf;

/// A workspace.
pub struct Project
{
    /// The project name.
    pub name: String,
    /// The file cache.
    pub cache: workspace::Cache,
    /// The sandboxing implementation.
    pub sandbox: Box<workspace::Sandbox>,
}

impl Project
{
    pub fn new(name: String, directory: PathBuf) -> Self {
        Project {
            name: name,
            cache: workspace::Cache::new(directory),
            sandbox: Box::new(workspace::sandbox::Basic),
        }
    }

    pub fn run(&mut self, command: job::Command)
        -> workspace::build::TaskOutput {
        self.sandbox.run(command, self.cache.directory())
    }
}


