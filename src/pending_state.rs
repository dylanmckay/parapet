use protocol;

/// The state of some connection that hasn't been promoted to a node.
#[derive(Clone, Debug)]
pub enum PendingState
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

impl PendingState
{
    pub fn is_complete(&self) -> bool {
        if let PendingState::Complete { .. } = *self { true } else { false }
    }
}

