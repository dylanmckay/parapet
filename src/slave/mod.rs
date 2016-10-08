pub use self::gaol::Gaol;

pub mod gaol;

use job;

/// A slave which can be used to execute commands in.
pub trait Slave
{
    fn run(&mut self, job::Command);
}

