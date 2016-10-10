use {Packet, PacketKind, Error};
use network::{PendingState, Connection};
use {network, protocol};

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

    pub fn advance_state(&mut self) -> Result<(), Error> {
        match self.state.clone() {
            PendingState::PendingPing => {
                let ping = protocol::Ping {
                    user_agent: ::user_agent(),
                    // TODO: randomise this data
                    data: vec![6, 2, 6, 1, 8, 8],
                };

                println!("sending ping");

                self.connection.send_packet(&Packet {
                    // FIXME: come up with a proper path
                    path: network::Path::empty(),
                    kind: PacketKind::Ping(ping.clone()),
                })?;

                self.state = PendingState::PendingPong { original_ping: ping };
            },
            PendingState::PendingJoinRequest => {
                self.connection.send_packet(&Packet {
                    path: network::Path::empty(),
                    kind: PacketKind::JoinRequest(protocol::JoinRequest),
                })?;
                println!("advancing from pending join request");

                self.state = PendingState::PendingJoinResponse;
            },
            _ => (), // we don't have to do anything.
        }

        Ok(())
    }

    pub fn process_incoming_data(&mut self) -> Result<(), Error> {
        match self.state.clone() {
            PendingState::PendingPing => (),
            PendingState::PendingPong { original_ping } => {
                if let Some(packet) = self.connection.receive_packet().unwrap() {
                    if let PacketKind::Pong(pong) = packet.kind {
                        println!("received pong");

                        // Check if the echoed data is correct.
                        if pong.data != original_ping.data {
                            return Err(Error::InvalidPong{
                                expected: original_ping.data.clone(),
                                received: pong.data,
                            });
                        }

                        // Ensure the protocol versions are compatible.
                        if !pong.user_agent.is_compatible(&original_ping.user_agent) {
                            // self.connection.terminate("protocol versions are not compatible")?;

                            // FIXME: Remove the connection.
                            unimplemented!();
                        }

                        self.state = PendingState::PendingJoinRequest;
                    } else {
                        return Err(Error::UnexpectedPacket { expected: "pong", received: packet })
                    }
                } else {
                    // we haven't received a full packet yet.
                }
            },
            PendingState::PendingJoinRequest  => (),
            PendingState::PendingJoinResponse => {
                if let Some(packet) = self.connection.receive_packet()? {
                    if let PacketKind::JoinResponse(join_response) = packet.kind {
                        self.state = PendingState::Complete { join_response: join_response };
                    } else {
                        return Err(Error::UnexpectedPacket { expected: "join response", received: packet })
                    }
                }
            },
            PendingState::Complete { .. } => {
                // nothing to do
            },
        }

        Ok(())
    }
}

