pub mod pending;
pub mod connected;

// FIXME: get rid of this
use super::*;

use mio;
use mio::tcp::*;
use slab::Slab;

use uuid::Uuid;
use std::time::Duration;

use std;
use proto;

const SERVER_TOKEN: mio::Token = mio::Token(usize::max_value() - 10);
const NEW_CONNECTION_TOKEN: mio::Token = mio::Token(usize::max_value() - 11);

// Flow:
//
// Client sends 'JoinRequest' to some node
// Server responds with 'JoinResponse'
// Client is now ready to act as server.

pub enum State
{
    Unconnected,
    Pending(pending::Node),
    /// We are now a connected node in the network.
    Connected {
        node: connected::Node,

        pending_connections: Slab<remote::pending::Node, mio::Token>,
    },
}

pub struct Parapet
{
    pub state: State,
    pub poll: mio::Poll,
}

impl Parapet
{
    /// Create a new network.
    pub fn new<A>(addr: A) -> Result<Self, Error>
        where A: std::net::ToSocketAddrs {
        let mut poll = mio::Poll::new()?;

        let listener = Parapet::bind(&mut poll, addr)?;
        let uuid = Uuid::new_v4();

        println!("assigning UUID {}", uuid);

        Ok(Parapet {
            state: State::Connected {
                node: connected::Node {
                    uuid: uuid,
                    listener: Some(listener),
                    network: Network::new(uuid),
                },
                pending_connections: Slab::with_capacity(1024),
            },
            poll: poll,
        })
    }

    /// Create a new tcp listener locally.
    fn bind<A>(poll: &mio::Poll, addr: A) -> Result<TcpListener, Error>
        where A: std::net::ToSocketAddrs {
        let mut addresses = addr.to_socket_addrs()?;
        let address = addresses.next().expect("could not resolve address");

        let listener = TcpListener::bind(&address)?;

        poll.register(&listener, SERVER_TOKEN, mio::Ready::readable(),
            mio::PollOpt::edge())?;

        Ok(listener)
    }

    /// Connect to an existing network.
    /// * `addr` - Any node on the network.
    pub fn connect<A>(addr: A) -> Result<Self, std::io::Error>
        where A: std::net::ToSocketAddrs {
        let mut addresses = addr.to_socket_addrs()?;
        let address = addresses.next().expect("could not resolve address");

        let stream = TcpStream::connect(&address)?;

        let poll = mio::Poll::new()?;
        poll.register(&stream, NEW_CONNECTION_TOKEN, mio::Ready::writable() | mio::Ready::readable(),
            mio::PollOpt::edge())?;

        let connection = Connection {
            token: NEW_CONNECTION_TOKEN,
            protocol: proto::wire::stream::Connection::new(stream, proto::wire::middleware::pipeline::default()),
        };

        Ok(Parapet {
            state: State::Pending(pending::Node::new(connection)),
            poll: poll,
        })
    }

    /// Attempts to advance the current state if possible.
    pub fn advance_new_connection_state(&mut self) -> Result<(), Error> {
        let mut current_state = State::Unconnected;
        std::mem::swap(&mut current_state, &mut self.state);

        self.state = match current_state {
            State::Pending(mut node) => {
                node.advance_state()?;

                if let PendingState::Complete { join_response } = node.state.clone() {
                    let listener = match Parapet::bind(&mut self.poll, SERVER_ADDRESS) {
                        Ok(listener) => Some(listener),
                        Err(Error::Io(e)) => match e.kind() {
                            std::io::ErrorKind::AddrInUse => {
                                println!("there is already something listening on port {} - we're not going to listen", SERVER_PORT);
                                None
                            },
                            _ => return Err(Error::Io(e)),
                        },
                        Err(e) => return Err(e),
                    };

                    let mut network: network::Network = join_response.network.into();
                    network.set_connection(&join_response.my_uuid, node.connection);

                    State::Connected {
                        node: connected::Node {
                            uuid: join_response.your_uuid,
                            listener: listener,
                            network: network,
                        },
                        pending_connections: Slab::with_capacity(1024),
                    }
                } else {
                    State::Pending(node)
                }
            },
            state => state,
        };

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            self.tick()?;
        }
    }

    pub fn tick(&mut self) -> Result<(), Error> {
        // Create storage for events
        let mut events = mio::Events::with_capacity(1024);

        self.advance_new_connection_state()?;
        self.poll.poll(&mut events, Some(Duration::from_millis(10))).unwrap();

        for event in events.iter() {
            match event.token() {
                // A pending connection.
                SERVER_TOKEN => {
                    if let State::Connected { ref mut node, ref mut pending_connections } = self.state {
                        let (socket, addr) = node.listener.as_mut().unwrap().accept()?;

                        println!("accepted connection from {:?}", addr);

                        let entry = pending_connections.vacant_entry().expect("ran out of connections");
                        let token = entry.index();

                        self.poll.register(&socket, token, mio::Ready::readable() | mio::Ready::writable(),
                            mio::PollOpt::edge())?;

                        entry.insert(remote::pending::Node::new(Connection {
                            token: token,
                            protocol: proto::wire::stream::Connection::new(socket, proto::wire::middleware::pipeline::default()),
                        }));
                    } else {
                        // We only start listening after we are successfully connected to the
                        // network.
                        unreachable!();
                    }
                },
                token => {
                    // TODO: check for `HUP` event.

                    match self.state {
                        State::Pending(ref mut proto_connection) => {
                            assert_eq!(token, NEW_CONNECTION_TOKEN);
                            if !event.kind().is_readable() {
                                continue;
                            }

                            match proto_connection.state.clone() {
                                PendingState::PendingPing => (),
                                PendingState::PendingPong { original_ping } => {
                                    if let Some(packet) = proto_connection.connection.receive_packet().unwrap() {
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
                                                // proto_connection.connection.terminate("protocol versions are not compatible")?;

                                                // FIXME: Remove the connection.
                                                unimplemented!();
                                            }

                                            proto_connection.state = PendingState::PendingJoinRequest;
                                        } else {
                                            return Err(Error::UnexpectedPacket { expected: "pong", received: packet })
                                        }
                                    } else {
                                        // we haven't received a full packet yet.
                                    }
                                },
                                PendingState::PendingJoinRequest  => (),
                                PendingState::PendingJoinResponse => {
                                    if let Some(packet) = proto_connection.connection.receive_packet()? {
                                        if let PacketKind::JoinResponse(join_response) = packet.kind {
                                            proto_connection.state = PendingState::Complete { join_response: join_response };
                                        } else {
                                            return Err(Error::UnexpectedPacket { expected: "join response", received: packet })
                                        }
                                    }
                                },
                                PendingState::Complete { .. } => {
                                    // nothing to do
                                },
                            }
                        },
                        State::Connected { ref mut node, ref mut pending_connections } => {
                            if !event.kind().is_readable() {
                                continue;
                            }

                            let packet = if let Some(mut pending_connection) = pending_connections.entry(token) {
                                pending_connection.get_mut().process_incoming_data(node)?;

                                let pending_connection = if pending_connection.get().is_complete() {
                                    pending_connection.remove()
                                } else {
                                    continue;
                                };

                                if let PendingState::Complete { ref join_response } = pending_connection.state {
                                    node.network.insert(network::Node {
                                        uuid: join_response.your_uuid.clone(),
                                        connection: Some(pending_connection.connection),
                                    });
                                    continue;
                                } else {
                                    continue;
                                }
                            } else if let Some(from_node) = node.network.lookup_token_mut(token) {
                                // we received a packet from an established node

                                if let Some(packet) = from_node.connection.as_mut().unwrap().receive_packet()? {
                                    packet
                                } else {
                                    continue;
                                }
                            } else {
                                unreachable!();
                            };

                            // Check if the packet is for us.
                            if packet.is_recipient(&node.uuid) {
                                println!("we got a packet");
                            } else {
                                // we need to forward this packet to the recipient
                                let next_hop_uuid = packet.path.next_hop(&node.uuid).unwrap();
                                let next_hop = node.network.get_mut(&next_hop_uuid).unwrap();

                                next_hop.connection.as_mut().unwrap().send_packet(&packet)?;
                            }
                        },
                        State::Unconnected => unreachable!(),
                    }
                },
            }
        }

        Ok(())
    }
}

