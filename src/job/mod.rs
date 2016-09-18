pub mod run;

use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Job
{
    pub uuid: Uuid,
    pub tasks: Vec<Task>,
}

#[derive(Clone, Debug)]
pub enum Task
{
    Run(Command),
}

#[derive(Clone, Debug)]
pub struct Command
{
    pub executable: String,
    pub arguments: Vec<String>,
}

