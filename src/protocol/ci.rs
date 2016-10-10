use ci;

use uuid::Uuid;

// A single command to execute.
define_composite_type!(Task {
    uuid: Uuid,
    command: Command
});

// A list of tasks to complete.
define_composite_type!(Work {
    uuid: Uuid,
    tasks: Vec<Task>
});

// A command to execute.
define_composite_type!(Command {
    executable: String,
    arguments: Vec<String>
});

// The result for a single task.
define_composite_type!(TaskResult {
    task: Task,
    output: Vec<u8>,
    result_code: i64
});

// Broadcasted by a node to tell everybody it has work available.
define_packet!(WorkAvailable);

// Broadcasted by a node to tell everybody it has ran out of work.
define_packet!(WorkComplete);

// Sent by a node to another node asking for work.
define_packet!(WorkRequest);

// Sent to from a node to another node, dishing out tasks for the
// other node to complete.
define_packet!(WorkResponse {
    work: Work
});

// Sent from a node to a node indicating that it has finished
// executing a piece of work.
define_packet!(WorkFinished {
    uuid: Uuid,
    tasks: Vec<TaskResult>
});

impl WorkResponse
{
    pub fn from_work(work: &ci::build::Work) -> Self {
        WorkResponse {
            work: Work {
                uuid: work.uuid.clone(),
                tasks: work.tasks.iter().map(|task| Task::from_task(task)).collect(),
            }
        }
    }
}

impl Task
{
    pub fn from_task(task: &ci::job::Task) -> Self {
        Task {
            uuid: task.uuid.clone(),
            command: Command::from_command(&task.command)
        }
    }
}

impl TaskResult
{
    pub fn from_task_result(task_result: &ci::build::TaskResult) -> Self {
        TaskResult {
            task: Task::from_task(&task_result.task),
            output: task_result.output.output.clone(),
            result_code: task_result.output.result_code,
        }
    }
}

impl Command
{
    pub fn from_command(command: &ci::job::Command) -> Self {
        Command {
            executable: command.executable.clone(),
            arguments: command.arguments.clone(),
        }
    }
}

impl Into<ci::build::Work> for WorkResponse
{
    fn into(self) -> ci::build::Work {
        ci::build::Work {
            uuid: self.work.uuid,
            tasks: self.work.tasks.iter().cloned().map(|t| t.into()).collect(),
        }
    }
}

impl Into<ci::job::Task> for Task
{
    fn into(self) -> ci::job::Task {
        ci::job::Task {
            uuid: self.uuid,
            command: self.command.into(),
        }
    }
}

impl Into<ci::build::TaskResult> for TaskResult
{
    fn into(self) -> ci::build::TaskResult {
        ci::build::TaskResult {
            task: self.task.into(),
            output: ci::build::TaskOutput {
                output: self.output,
                result_code: self.result_code,
            },
        }
    }
}

impl Into<ci::job::Command> for Command {
    fn into(self) -> ci::job::Command {
        ci::job::Command {
            executable: self.executable,
            arguments: self.arguments,
        }
    }
}

