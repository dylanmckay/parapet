use {Packet, Connection, ProtoConnection, ProtoState, Error, ConnectedNode, Node};
use protocol;

use uuid::Uuid;

/// A new node connecting to our current network.
pub enum ProtoNode
{
    Pending(ProtoConnection),
    Completed
}

impl ProtoNode
{
    pub fn new(connection: Connection) -> Self {
        ProtoNode::Pending(ProtoConnection::new(connection))
    }

    pub fn process_incoming_data(&mut self, connected_node: &mut ConnectedNode) -> Result<(), Error> {
        let mut tmp = ProtoNode::Completed;
        ::std::mem::swap(&mut tmp, self);

        *self = if let ProtoNode::Pending(mut proto_connection) = tmp {
            match proto_connection.state.clone() {
                ProtoState::PendingPing => {
                    if let Some(packet) = proto_connection.connection.receive_packet()? {
                        if let Packet::Ping(ping) = packet {
                            println!("received ping, responding with pong");

                            let pong = protocol::Pong {
                                user_agent: ::user_agent(),
                                data: ping.data.clone(),
                            };

                            proto_connection.connection.send_packet(&Packet::Pong(pong.clone()))?;

                            proto_connection.state = ProtoState::PendingJoinRequest;
                        } else {
                            return Err(Error::UnexpectedPacket { expected: "ping", received: packet });
                        }
                    }

                    ProtoNode::Pending(proto_connection)
                },
                ProtoState::PendingJoinRequest => {
                    if let Some(packet) = proto_connection.connection.receive_packet()? {
                        if let Packet::JoinRequest(..) = packet {
                            let new_node_uuid = Uuid::new_v4();

                            connected_node.network.insert(Node {
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
                            new_node.connection.as_mut().unwrap().send_packet(&Packet::JoinResponse(join_response.clone()))?;

                            proto_connection.state = ProtoState::Complete { join_response: join_response };

                            ProtoNode::Completed
                        } else {
                            return Err(Error::UnexpectedPacket { expected: "join request", received: packet });
                        }
                    } else {
                        ProtoNode::Pending(proto_connection)
                    }
                },
                _ => ProtoNode::Pending(proto_connection),
            }
        } else {
            ProtoNode::Completed
        };

        Ok(())
    }
}

