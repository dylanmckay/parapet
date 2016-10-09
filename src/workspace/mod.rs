pub use self::strategy::Strategy;
pub use self::cache::Cache;
pub use self::gaol::Gaol;
pub use self::basic::Basic;

pub mod strategy;
pub mod cache;
pub mod gaol;
pub mod basic;

use job;
use std::path::PathBuf;

/// A workspace to run commands in.
pub trait Sandbox : Send
{
    fn run(&mut self, job::Command) -> job::run::TaskOutput;
}

/// A directory-based sandbox.
pub trait DirectoryBased : Sandbox
{
    fn from_directory(directory: PathBuf) -> Self;
}

