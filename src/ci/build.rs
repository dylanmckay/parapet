use ci::{Task, Sandbox};

use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::collections::VecDeque;

use uuid::Uuid;

/// A piece of work dished out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Work
{
    pub uuid: Uuid,
    pub tasks: VecDeque<Task>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkOutput
{
    pub work: Work,
    pub task_results: Vec<TaskResult>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskResult
{
    pub task: Task,
    pub output: TaskOutput,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskOutput
{
    pub output: Vec<u8>,
    pub result_code: i64,
}

impl TaskOutput
{
    pub fn is_successful(&self) -> bool {
        self.result_code == 0
    }
}

pub fn work(work: Work, mut sandbox: Box<Sandbox>, sender: mpsc::Sender<WorkOutput>) {
    thread::spawn(move || {
        let mut results = Vec::new();

        for task in work.tasks.iter() {
            let result = self::task(task.clone(), &mut sandbox);
            results.push(result.clone());

            if !result.output.is_successful() { break };
        }

        let work_output = WorkOutput {
            work: work,
            task_results: results,
        };

        sender.send(work_output).ok();
    });
}

pub fn task(task: Task, sandbox: &mut Box<Sandbox>) -> TaskResult
{
    let path = format!("nameless-work-{}", Uuid::new_v4());
    let task_output = sandbox.run(task.command.clone(), &Path::new(&path));

    TaskResult {
        task: task,
        output: task_output,
    }
}

