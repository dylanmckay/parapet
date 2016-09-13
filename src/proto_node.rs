use {Packet, Connection, ProtoConnection, ProtoState, Error, ConnectedNode};
use protocol;

use uuid::Uuid;

/// A new node connecting to our current network.
pub struct ProtoNode(pub ProtoConnection);

impl ProtoNode
{
    pub fn new(connection: Connection) -> Self {
        ProtoNode(ProtoConnection::new(connection))
    }

    pub fn process_incoming_data(&mut self, connected_node: &ConnectedNode) -> Result<(), Error> {
        match self.0.state {
            ProtoState::PendingPing => {
                if let Some(packet) = self.0.connection.receive_packet()? {
                    if let Packet::Ping(ping) = packet {
                        let pong = protocol::Pong {
                            user_agent: ::user_agent(),
                            data: ping.data.clone(),
                        };

                        self.0.connection.send_packet(&Packet::Pong(pong.clone()))?;

                        self.0.state = ProtoState::PendingJoinRequest;
                    } else {
                        return Err(Error::UnexpectedPacket { expected: "ping", received: packet });
                    }
                }
            },
            ProtoState::PendingJoinRequest => {
                if let Some(packet) = self.0.connection.receive_packet()? {
                    if let Packet::JoinRequest(..) = packet {
                        let network = protocol::Network::from_network(&connected_node.network);

                        let join_response = protocol::JoinResponse {
                            your_uuid: Uuid::new_v4(),
                            my_uuid: connected_node.uuid.clone(),
                            network: network,
                        };

                        self.0.connection.send_packet(&Packet::JoinResponse(join_response.clone()))?;

                        self.0.state = ProtoState::Complete { join_response: join_response };
                    } else {
                        return Err(Error::UnexpectedPacket { expected: "join request", received: packet });
                    }
                }
            },
            ProtoState::Complete { .. } => (),
            _ => (),
        }

        Ok(())
    }
}

