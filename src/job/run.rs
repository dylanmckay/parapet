use job::Work;
use Sandbox;

use job;

use std::thread;
use std::sync::mpsc;

#[derive(Clone, Debug)]
pub struct WorkOutput
{
    pub work: Work,
    pub task_results: Vec<TaskResult>,
}

#[derive(Clone, Debug)]
pub struct TaskResult
{
    pub task: job::Task,
    pub output: TaskOutput,
}

#[derive(Clone, Debug)]
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

pub fn task(task: job::Task, sandbox: &mut Box<Sandbox>) -> TaskResult
{
    let task_output = sandbox.run(task.command.clone());

    TaskResult {
        task: task,
        output: task_output,
    }
}

