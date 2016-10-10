use {Packet, PacketKind, Connection, Error};
use comm::PendingState;
use network;
use comm::local;
use protocol;

use uuid::Uuid;


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

    pub fn is_complete(&self) -> bool {
        if let PendingState::Complete { .. } = self.state { true } else { false }
    }

    pub fn process_incoming_data(&mut self, connected_node: &mut local::connected::Node) -> Result<(), Error> {
        match self.state.clone() {
            PendingState::PendingPing => {
                if let Some(packet) = self.connection.receive_packet()? {
                    if let PacketKind::Ping(ping) = packet.kind {
                        println!("received ping, responding with pong");

                        let pong = protocol::Pong {
                            user_agent: ::user_agent(),
                            data: ping.data.clone(),
                        };

                        self.connection.send_packet(&Packet {
                            path: network::Path::empty(),
                            kind: PacketKind::Pong(pong.clone()),
                        })?;

                        self.state = PendingState::PendingJoinRequest;
                    } else {
                        return Err(Error::UnexpectedPacket { expected: "ping", received: packet });
                    }
                }
            },
            PendingState::PendingJoinRequest => {
                if let Some(packet) = self.connection.receive_packet()? {
                    if let PacketKind::JoinRequest(..) = packet.kind {
                        let new_node_uuid = Uuid::new_v4();

                        println!("received join request, responding (assigning UUID {})", new_node_uuid);

                        let network = protocol::Network::from_network(&connected_node.network);

                        let join_response = protocol::JoinResponse {
                            your_uuid: new_node_uuid,
                            my_uuid: connected_node.uuid.clone(),
                            network: network,
                        };

                        self.connection.send_packet(&Packet {
                            path: network::Path::empty(),
                            kind: PacketKind::JoinResponse(join_response.clone()),
                        })?;

                        self.state = PendingState::Complete { join_response: join_response };
                    } else {
                        return Err(Error::UnexpectedPacket { expected: "join request", received: packet });
                    }
                }
            },
            _ => (),
        }

        Ok(())
    }
}

