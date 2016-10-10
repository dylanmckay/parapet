pub use self::basic::Basic;

pub mod basic;

use ci::{build, Command};

use std::path::Path;

/// A ci to run commands in.
pub trait Sandbox : Send
{
    fn run(&mut self, command: Command, working_dir: &Path)
        -> build::TaskOutput;
}

