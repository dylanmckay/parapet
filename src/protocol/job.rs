use {workspace, job};

use uuid::Uuid;

define_composite_type!(Task {
    uuid: Uuid,
    command: Command
});

define_composite_type!(Work {
    uuid: Uuid,
    tasks: Vec<Task>
});

define_composite_type!(WorkResult {
    uuid: Uuid,
    task_results: Vec<TaskResult>
});

define_composite_type!(Command {
    executable: String,
    arguments: Vec<String>
});

define_composite_type!(TaskResult {
    task: Task,
    output: Vec<u8>,
    result_code: i64
});

define_packet!(WorkRequest {
    work: Work
});

define_packet!(WorkResponse {
    uuid: Uuid,
    tasks: Vec<TaskResult>
});

impl WorkRequest
{
    pub fn from_work(work: &workspace::build::Work) -> Self {
        WorkRequest {
            work: Work {
                uuid: work.uuid.clone(),
                tasks: work.tasks.iter().map(|task| Task::from_task(task)).collect(),
            }
        }
    }
}

impl Task
{
    pub fn from_task(task: &job::Task) -> Self {
        Task {
            uuid: task.uuid.clone(),
            command: Command::from_command(&task.command)
        }
    }
}

impl TaskResult
{
    pub fn from_task_result(task_result: &workspace::build::TaskResult) -> Self {
        TaskResult {
            task: Task::from_task(&task_result.task),
            output: task_result.output.output.clone(),
            result_code: task_result.output.result_code,
        }
    }
}

impl Command
{
    pub fn from_command(command: &job::Command) -> Self {
        Command {
            executable: command.executable.clone(),
            arguments: command.arguments.clone(),
        }
    }
}

impl Into<workspace::build::Work> for WorkRequest
{
    fn into(self) -> workspace::build::Work {
        workspace::build::Work {
            uuid: self.work.uuid,
            tasks: self.work.tasks.iter().cloned().map(|t| t.into()).collect(),
        }
    }
}

impl Into<job::Task> for Task
{
    fn into(self) -> job::Task {
        job::Task {
            uuid: self.uuid,
            command: self.command.into(),
        }
    }
}

impl Into<job::Command> for Command {
    fn into(self) -> job::Command {
        job::Command {
            executable: self.executable,
            arguments: self.arguments,
        }
    }
}

