pub use self::strategy::Strategy;
pub use self::gaol::Gaol;
pub use self::basic::Basic;

pub mod strategy;
pub mod gaol;
pub mod basic;

use job;
use std::path::PathBuf;

/// A workspace to run commands in.
pub trait Workspace : Send
{
    fn run(&mut self, job::Command) -> job::run::TaskOutput;
}

/// A directory-based workspace.
pub trait DirectoryBased : Workspace
{
    fn from_directory(directory: PathBuf) -> Self;
}

