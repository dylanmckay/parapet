use {Connection, Error};

/// A new connection which has not yet identified itself as a node.
pub struct ProtoConnection
{
    pub connection: Connection,
    pub state: State,
}

pub enum State
{
    /// New connection, no data send/received.
    New,
}

impl ProtoConnection
{
    pub fn new(connection: Connection) -> Self {
        ProtoConnection {
            connection: connection,
            state: State::New,
        }
    }

    pub fn process_incoming_data(&self) -> Result<(), Error> {
        unimplemented!();
    }
}

