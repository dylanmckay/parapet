use job;

use uuid::Uuid;

define_composite_type!(Run {
    executable: String,
    arguments: Vec<String>
});

define_packet_kind!(Task: u8 {
    0x00 => Run
});

define_composite_type!(TaskOutput {
    task: Task,
    output: Vec<u8>,
    result_code: i64
});

define_packet!(JobRequest {
    uuid: Uuid,
    tasks: Vec<Task>
});

define_packet!(JobResponse {
    uuid: Uuid,
    tasks: Vec<TaskOutput>
});

impl JobRequest
{
    pub fn from_job(job: &job::Job) -> Self {
        JobRequest {
            uuid: job.uuid.clone(),
            tasks: job.tasks.iter().map(|task| Task::from_task(task)).collect(),
        }
    }
}

impl Task
{
    pub fn from_task(task: &job::Task) -> Self {
        match *task {
            job::Task::Run(ref command) => Task::Run(Run::from_command(command)),
        }
    }
}

impl TaskOutput
{
    pub fn from_task_output(task_output: &job::run::TaskOutput) -> Self {
        TaskOutput {
            task: Task::from_task(&task_output.task),
            output: task_output.output.clone(),
            result_code: task_output.result_code,
        }
    }
}

impl Run
{
    pub fn from_command(command: &job::Command) -> Self {
        Run {
            executable: command.executable.clone(),
            arguments: command.arguments.clone(),
        }
    }
}

impl Into<job::Job> for JobRequest
{
    fn into(self) -> job::Job {
        job::Job {
            uuid: self.uuid,
            tasks: self.tasks.iter().cloned().map(|t| t.into()).collect(),
        }
    }
}

impl Into<job::Task> for Task
{
    fn into(self) -> job::Task {
        match self {
            Task::Run(command) => job::Task::Run(command.into()),
        }
    }
}

impl Into<job::Command> for Run {
    fn into(self) -> job::Command {
        job::Command {
            executable: self.executable,
            arguments: self.arguments,
        }
    }
}

