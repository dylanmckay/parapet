use Job;
use job;

use std::thread;
use std::sync::mpsc;

#[derive(Clone, Debug)]
pub struct JobOutput
{
    pub job: Job,
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

pub fn job(job: Job, sender: mpsc::Sender<JobOutput>) {
    thread::spawn(move || {
        let mut results = Vec::new();

        for task in job.tasks.iter() {
            let result = self::task(task.clone());
            results.push(result.clone());

            if !result.output.is_successful() { break };
        }

        let job_output = JobOutput {
            job: job,
            task_results: results,
        };

        sender.send(job_output).ok();
    });
}

pub fn task(task: job::Task) -> TaskResult
{
    match task.clone() {
        job::Task::Run(command) => {
            use slave::Slave;

            let mut slave = ::slave::Basic::new("/tmp/foo");
            let task_output = slave.run(command);

            TaskResult {
                task: task,
                output: task_output,
            }
        },
    }
}

