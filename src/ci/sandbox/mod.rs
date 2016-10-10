pub use self::basic::Basic;

pub mod basic;

use job;
use ci::build;

use std::path::Path;

/// A ci to run commands in.
pub trait Sandbox : Send
{
    fn run(&mut self, command: job::Command, working_dir: &Path)
        -> build::TaskOutput;
}

