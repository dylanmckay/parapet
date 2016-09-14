use job;

define_composite_type!(Run {
    executable: String,
    arguments: Vec<String>
});

define_packet_kind!(Task: u8 {
    0x00 => Run
});

define_packet!(JobRequest {
    tasks: Vec<Task>
});

impl JobRequest
{
    pub fn from_job(job: &job::Job) -> Self {
        JobRequest {
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

impl Run
{
    pub fn from_command(command: &job::Command) -> Self {
        Run {
            executable: command.executable.clone(),
            arguments: command.arguments.clone(),
        }
    }
}

