use Connection;
use protocol;

/// A new connection which has not yet identified itself as a node.
#[derive(Clone, Debug)]
pub enum State
{
    /// We just connected and need to send a 'Ping'.
    PendingPing,

    /// We sent a `Ping` and are awaiting a `Pong`.
    PendingPong {
        /// The original ping that we sent.
        original_ping: protocol::Ping,
    },

    /// Pong matched original data, we now need to send a `JoinRequest`.
    PendingJoinRequest,
    /// We sent a `JoinRequest` and are awaiting a response.
    PendingJoinResponse,

    Complete {
        join_response: protocol::JoinResponse,
    },
}

#[derive(Debug)]
pub struct Node
{
    pub state: State,
    pub connection: Connection,
}

impl Node
{
    pub fn new(connection: Connection) -> Self {
        Node {
            state: State::PendingPing,
            connection: connection,
        }
    }
}

