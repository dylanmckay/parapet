use {PendingState, Connection};

#[derive(Debug)]
pub struct Node
{
    pub state: PendingState,
    pub connection: Connection,
}

impl Node
{
    pub fn new(connection: Connection) -> Self {
        Node {
            state: PendingState::PendingPing,
            connection: connection,
        }
    }
}

