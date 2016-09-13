use {Packet, Connection, ProtoConnection, ProtoState, Error};

/// A new node connecting to our current network.
pub struct ProtoNode(pub ProtoConnection);

impl ProtoNode
{
    pub fn new(connection: Connection) -> Self {
        ProtoNode(ProtoConnection::new(connection))
    }

    pub fn process_incoming_data(&mut self) -> Result<(), Error> {
        match self.0.state {
            ProtoState::PendingPing => {
                // if let Some(packet) = connection.receive_packet()? {
                //     if let Packet::Ping(ping) = packet {
                //         self.0 = ProtoState::PendingJoinRequest { connection: connection };
                //     }
                // }
            },
            ProtoState::PendingPong { .. } => {
            },
            ProtoState::PendingJoinRequest => (),
            ProtoState::PendingJoinResponse => {
            },
            ProtoState::Complete { .. } => (),
        }

        Ok(())
    }
}

