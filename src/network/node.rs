use network::{remote, Connection, Status};
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
    pub fn can_ask_for_work(&self) -> bool {
        if let Status::Remote(ref s) = self.status {
            if let remote::status::Work::Available { have_asked_for_work } = s.work { !have_asked_for_work } else { false }
        } else {
            false
        }
    }
}

