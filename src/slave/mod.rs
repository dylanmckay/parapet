pub use self::gaol::Gaol;
pub use self::basic::Basic;

pub mod gaol;
pub mod basic;

use job;

/// A slave which can be used to execute commands in.
pub trait Slave
{
    fn run(&mut self, job::Command) -> job::run::TaskOutput;
}

