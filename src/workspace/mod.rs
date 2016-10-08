pub use self::gaol::Gaol;
pub use self::basic::Basic;

pub mod gaol;
pub mod basic;

use job;

/// A workspace to run commands in.
pub trait Workspace
{
    fn run(&mut self, job::Command) -> job::run::TaskOutput;
}

