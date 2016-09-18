use Job;
use job;

use std::thread;
use std::sync::mpsc;
use std::process;

#[derive(Clone, Debug)]
pub struct Output
{
    pub job: Job,
    pub task_outputs: Vec<TaskOutput>,
}

#[derive(Clone, Debug)]
pub struct TaskOutput
{
    pub task: job::Task,
    pub output: Vec<u8>,
    pub result_code: i64,
}

impl TaskOutput
{
    pub fn is_successful(&self) -> bool {
        self.result_code == 0
    }
}

pub fn job(job: Job, sender: mpsc::Sender<Output>) {
    thread::spawn(move || {
        let mut outputs = Vec::new();

        for task in job.tasks.iter() {
            let output = self::task(task.clone());
            outputs.push(output.clone());

            if !output.is_successful() { break };
        }

        let job_output = Output {
            job: job,
            task_outputs: outputs,
        };

        sender.send(job_output).ok();
    });
}

pub fn task(task: job::Task) -> TaskOutput
{
    match task.clone() {
        job::Task::Run(command) => {
            let output = process::Command::new(&command.executable)
                .args(&command.arguments)
                .output()
                .expect("could not spawn command");

            TaskOutput {
                task: task,
                output: output.stdout,
                result_code: match output.status.code() {
                    Some(code) => code as _,
                    None => 0,
                },
            }
        },
    }
}

