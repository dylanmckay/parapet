use workspace::{self, Sandbox};

use std::marker::PhantomData;
use std::path::PathBuf;
use std::fs;

use uuid::Uuid;

/// A strategy for creating workspaces.
pub trait Strategy
{
    fn create_workspace(&mut self, name: &str) -> Box<Sandbox>;
}

/// A strategy which works in a directory.
pub struct InDirectory<W: Sandbox>
{
    directory: PathBuf,
    phantom: PhantomData<W>,
}

impl<W: workspace::DirectoryBased> InDirectory<W>
{
    pub fn new<P>(directory: P) -> Self
        where P: Into<PathBuf> {
        InDirectory {
            directory: directory.into(),
            phantom: PhantomData,
        }
    }
}

impl<W: workspace::DirectoryBased+'static> Strategy for InDirectory<W>
{
    fn create_workspace(&mut self, name: &str) -> Box<Sandbox> {
        if !self.directory.exists() {
            fs::create_dir_all(&self.directory).expect("could not create workspace directory");
        }

        let subdirectory = format!("{}-{}", name, Uuid::new_v4());
        Box::new(W::from_directory(self.directory.join(subdirectory)))
    }
}

