use Workspace;
use workspace;
use job;

use std::path::PathBuf;
use std::{process, fs};

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

impl Workspace for Basic
{
    fn run(&mut self, command: job::Command) -> job::run::TaskOutput {
        if !self.directory.exists() {
            fs::create_dir(&self.directory).expect("could not create workspace directory");
        }

        let output = process::Command::new(&command.executable)
            .args(&command.arguments)
            .current_dir(&self.directory)
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

impl workspace::DirectoryBased for Basic
{
    fn from_directory(directory: PathBuf) -> Self {
        Basic::new(directory)
    }
}

