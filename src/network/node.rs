use network::{Connection, Status};
use uuid::Uuid;

#[derive(Debug)]
pub struct Node
{
    pub uuid: Uuid,
    pub connection: Option<Connection>,
    pub status: Status,
}

impl Node
{
    pub fn has_work_available(&self) -> bool {
        if let Status::Remote(ref s) = self.status { s.work_available } else { false }
    }
}

