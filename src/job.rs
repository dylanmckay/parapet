use uuid::Uuid;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Job
{
    pub uuid: Uuid,
    pub tasks: Vec<Task>,
}

#[derive(Clone, Debug)]
pub struct Task
{
    pub uuid: Uuid,
    pub command: Command,
}

#[derive(Clone, Debug)]
pub struct Command
{
    pub executable: String,
    pub arguments: Vec<String>,
}

/// A piece of work dished out.
#[derive(Clone, Debug)]
pub struct Work
{
    pub uuid: Uuid,
    pub tasks: VecDeque<Task>,
}

