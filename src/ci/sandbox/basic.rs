use ci::{build, Command, Sandbox};

use std::path::Path;
use std::{process, fs};

pub struct Basic;

impl Sandbox for Basic
{
    fn run(&mut self, command: Command, working_dir: &Path) -> build::TaskOutput {
        if !working_dir.exists() {
            fs::create_dir_all(&working_dir).expect("could not create ci directory");
        }

        let output = process::Command::new(&command.executable)
            .args(&command.arguments)
            .current_dir(working_dir)
            .output()
            .expect("could not spawn command");

        let output = build::TaskOutput {
            // FIXME: grab stderr
            output: output.stdout,
            result_code: match output.status.code() {
                Some(code) => code as _,
                None => 0,
            },
        };

        output
    }
}

