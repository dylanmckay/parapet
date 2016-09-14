use {PendingState, Packet, PacketKind, Connection, Error};
use network;
use local;
use protocol;

use uuid::Uuid;


#[derive(Debug)]
pub struct RemotePendingNode
{
    pub state: PendingState,
    pub connection: Connection,
}

impl RemotePendingNode
{
    pub fn new(connection: Connection) -> Self {
        RemotePendingNode {
            state: PendingState::PendingPing,
            connection: connection,
        }
    }
}

/// A new node connecting to our current network.
pub enum Node
{
    Pending(RemotePendingNode),
    Completed
}

impl Node
{
    pub fn new(connection: Connection) -> Self {
        Node::Pending(RemotePendingNode::new(connection))
    }

    pub fn process_incoming_data(&mut self, connected_node: &mut local::connected::Node) -> Result<(), Error> {
        let mut tmp = Node::Completed;
        ::std::mem::swap(&mut tmp, self);

        *self = if let Node::Pending(mut proto_connection) = tmp {
            match proto_connection.state.clone() {
                PendingState::PendingPing => {
                    if let Some(packet) = proto_connection.connection.receive_packet()? {
                        if let PacketKind::Ping(ping) = packet.kind {
                            println!("received ping, responding with pong");

                            let pong = protocol::Pong {
                                user_agent: ::user_agent(),
                                data: ping.data.clone(),
                            };

                            proto_connection.connection.send_packet(&Packet {
                                path: network::Path::empty(),
                                kind: PacketKind::Pong(pong.clone()),
                            })?;

                            proto_connection.state = PendingState::PendingJoinRequest;
                        } else {
                            return Err(Error::UnexpectedPacket { expected: "ping", received: packet });
                        }
                    }

                    Node::Pending(proto_connection)
                },
                PendingState::PendingJoinRequest => {
                    if let Some(packet) = proto_connection.connection.receive_packet()? {
                        if let PacketKind::JoinRequest(..) = packet.kind {
                            let new_node_uuid = Uuid::new_v4();

                            connected_node.network.insert(network::Node {
                                uuid: new_node_uuid.clone(),
                                connection: Some(proto_connection.connection),
                            });

                            println!("received join request, responding (assigning UUID {})", new_node_uuid);

                            let network = protocol::Network::from_network(&connected_node.network);
                            println!("describe: {:?}", network);

                            let join_response = protocol::JoinResponse {
                                your_uuid: new_node_uuid,
                                my_uuid: connected_node.uuid.clone(),
                                network: network,
                            };

                            let mut new_node = connected_node.network.get_mut(&new_node_uuid).unwrap();
                            new_node.connection.as_mut().unwrap().send_packet(&Packet {
                                path: network::Path::empty(),
                                kind: PacketKind::JoinResponse(join_response.clone()),
                            })?;

                            proto_connection.state = PendingState::Complete { join_response: join_response };

                            Node::Completed
                        } else {
                            return Err(Error::UnexpectedPacket { expected: "join request", received: packet });
                        }
                    } else {
                        Node::Pending(proto_connection)
                    }
                },
                _ => Node::Pending(proto_connection),
            }
        } else {
            Node::Completed
        };

        Ok(())
    }
}

