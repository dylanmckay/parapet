use Connection;
use protocol;

/// A new connection which has not yet identified itself as a node.
#[derive(Clone, Debug)]
pub enum ProtoState
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
pub struct ProtoConnection
{
    pub state: ProtoState,
    pub connection: Connection,
}

impl ProtoConnection
{
    pub fn new(connection: Connection) -> Self {
        ProtoConnection {
            state: ProtoState::PendingPing,
            connection: connection,
        }
    }
}

