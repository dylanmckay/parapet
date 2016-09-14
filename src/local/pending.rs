use {Packet, PacketKind, PendingState, Connection, Error};
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
}

