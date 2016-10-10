use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job
{
    pub uuid: Uuid,
    pub tasks: Vec<Task>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task
{
    pub uuid: Uuid,
    pub command: Command,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command
{
    pub executable: String,
    pub arguments: Vec<String>,
}

