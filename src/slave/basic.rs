use Slave;
use job;

use std::path::PathBuf;
use std::process;

pub struct Basic {
    directory: PathBuf,
}

impl Basic
{
    pub fn new<P>(directory: P) -> Self
        where P: Into<PathBuf> {
        Basic {
            directory: directory.into(),
        }
    }
}

impl Slave for Basic
{
    fn run(&mut self, command: job::Command) -> job::run::TaskOutput {
        let output = process::Command::new(&command.executable)
            .args(&command.arguments)
            .output()
            .expect("could not spawn command");

        job::run::TaskOutput {
            // FIXME: grab stderr
            output: output.stdout,
            result_code: match output.status.code() {
                Some(code) => code as _,
                None => 0,
            },
        }
    }
}

