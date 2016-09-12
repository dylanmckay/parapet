use Network;
use {Connection, Error};
use protocol;

use uuid::Uuid;

/// A new connection which has not yet identified itself as a node.
pub enum ProtoConnection
{
    /// We just connected and need to send a 'Ping'.
    PendingPing {
        connection: Connection,
    },
    /// We sent a `Ping` and are awaiting a `Pong`.
    PendingPong {
        connection: Connection,

        /// The original ping that we sent.
        original_ping: protocol::Ping,
    },
    /// Pong matched original data, we now need to send a `JoinRequest`.
    PendingJoinRequest {
        connection: Connection,
    },
    /// We sent a `JoinRequest` and are awaiting a response.
    PendingJoinResponse {
        connection: Connection,
    },
    Complete {
        your_uuid: Uuid,
        my_uuid: Uuid,
        network: Network,
    },
}

impl ProtoConnection
{
    pub fn new(connection: Connection) -> Self {
        ProtoConnection::PendingPing {
            connection: connection,
        }
    }

    pub fn process_incoming_data(&self) -> Result<(), Error> {
        unimplemented!();
    }
}

