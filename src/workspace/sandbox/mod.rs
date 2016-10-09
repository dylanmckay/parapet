pub use self::basic::Basic;

pub mod basic;

use job;

use std::path::Path;

/// A workspace to run commands in.
pub trait Sandbox : Send
{
    fn run(&mut self, command: job::Command, working_dir: &Path)
        -> job::run::TaskOutput;
}

