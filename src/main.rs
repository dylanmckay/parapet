#![feature(question_mark)]
#![feature(associated_consts)]
#![feature(const_fn)]
#![feature(conservative_impl_trait)]

extern crate mio;
extern crate slab;
extern crate uuid;
extern crate byteorder;
extern crate graphsearch;
extern crate clap;
#[macro_use]
extern crate protocol as proto;

pub use self::proto_connection::{ProtoConnection, ProtoState};
pub use self::proto_node::ProtoNode;
pub use self::connection::*;
pub use self::network::{Network, Node, Edge};
pub use self::error::Error;
pub use self::protocol::Packet;
pub use self::path::Path;

pub mod proto_connection;
pub mod proto_node;
pub mod connection;
pub mod network;
pub mod error;
pub mod protocol;
pub mod path;

use mio::tcp::*;
use slab::Slab;

use uuid::Uuid;

const SERVER_TOKEN: mio::Token = mio::Token(usize::max_value() - 10);

const SERVER_PORT: u16 = 53371;
const SERVER_ADDRESS: (&'static str, u16) = ("127.0.0.1", SERVER_PORT);

const CLIENT_NAME: &'static str = "vanilla";
const CLIENT_VERSION: &'static str = env!("CARGO_PKG_VERSION");

const PROTOCOL_MAJOR: u16 = 0;
const PROTOCOL_REVISION: u16 = 0;

const DESCRIPTION: &'static str = "
    If you pass an address, it will connect to an existing node on
    some network, otherwise a new network will be created.
";

fn user_agent() -> protocol::UserAgent {
    protocol::UserAgent {
        client: format!("{} v{}", CLIENT_NAME, CLIENT_VERSION),
        protocol_major: PROTOCOL_MAJOR,
        protocol_revision: PROTOCOL_REVISION,
    }
}

pub struct ConnectedNode
{
    pub uuid: Uuid,
    pub listener: TcpListener,

    /// The network we are apart of.
    pub network: Network,
}

// Flow:
//
// Client sends 'JoinRequest' to some node
// Server responds with 'JoinResponse'
// Client is now ready to act as server.

pub enum State
{
    Unconnected,
    Pending(ProtoConnection),
    /// We are now a connected node in the network.
    Connected {
        node: ConnectedNode,

        pending_connections: Slab<ProtoNode, mio::Token>,
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

        Ok(Parapet {
            state: State::Connected {
                node: ConnectedNode {
                    uuid: Uuid::new_v4(),
                    listener: listener,
                    network: Network::new(),
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

        // FIXME: We need a better way to track tokens
        let tmp_token = mio::Token(500);

        let poll = mio::Poll::new()?;
        poll.register(&stream, tmp_token, mio::Ready::writable(),
            mio::PollOpt::edge())?;

        Ok(Parapet {
            state: State::Pending(ProtoConnection::new(Connection {
                token: tmp_token,
                protocol: proto::wire::stream::Connection::new(stream, proto::wire::middleware::pipeline::default()),
            })),
            poll: poll,
        })
    }

    /// Attempts to advance the current state if possible.
    pub fn advance(&mut self) -> Result<(), Error> {
        self.mutate_state(|parapet, state|
            if let State::Pending(mut proto_connection) = state {
                match proto_connection.state.clone() {
                    ProtoState::PendingPing => {
                        let ping = protocol::Ping {
                            user_agent: user_agent(),
                            // TODO: randomise this data
                            data: vec![6, 2, 6, 1, 8, 8],
                        };

                        proto_connection.connection.send_packet(&Packet::Ping(ping.clone()))?;
                        proto_connection.state = ProtoState::PendingPong { original_ping: ping };

                        Ok(State::Pending(proto_connection))
                    },
                    ProtoState::PendingJoinRequest => {
                        proto_connection.connection.send_packet(&Packet::JoinRequest(protocol::JoinRequest))?;

                        proto_connection.state = ProtoState::PendingJoinResponse;
                        Ok(State::Pending(proto_connection))
                    },
                    ProtoState::Complete { join_response } => {
                        let mut network: Network = join_response.network.into();
                        let listener = Parapet::bind(&mut parapet.poll, SERVER_ADDRESS)?;

                        network.set_connection(&join_response.my_uuid, proto_connection.connection);

                        Ok(State::Connected {
                            node: ConnectedNode {
                                uuid: join_response.your_uuid,
                                listener: listener,
                                network: network,
                            },
                            pending_connections: Slab::with_capacity(1024),
                        })
                    },
                    _ => Ok(State::Pending(proto_connection)), // we don't have to do anything.
                }
            } else {
                Ok(state)
            }
        )
    }

    pub fn run(&mut self) -> Result<(), Error> {
        // Create storage for events
        let mut events = mio::Events::with_capacity(1024);

        loop {
            self.poll.poll(&mut events, None).unwrap();

            self.advance()?;

            for event in events.iter() {
                match event.token() {
                    // A pending connection.
                    SERVER_TOKEN => {
                        if let State::Connected { ref mut node, ref mut pending_connections } = self.state {
                            let (socket, addr) = node.listener.accept()?;

                            println!("accepted connection from {:?}", addr);

                            let entry = pending_connections.vacant_entry().expect("ran out of connections");
                            let token = entry.index();

                            self.poll.register(&socket, token, mio::Ready::readable(),
                                mio::PollOpt::edge())?;

                            entry.insert(ProtoNode(ProtoConnection::new(Connection {
                                token: token,
                                protocol: proto::wire::stream::Connection::new(socket, proto::wire::middleware::pipeline::default()),
                            })));
                        } else {
                            // We only start listening after we are successfully connected to the
                            // network.
                            unreachable!();
                        }
                    },
                    token => {
                        match self.state {
                            State::Pending(ref mut proto_connection) => match proto_connection.state.clone() {
                                ProtoState::PendingPing => (),
                                ProtoState::PendingPong { original_ping } => {
                                    if let Some(packet) = proto_connection.connection.receive_packet()? {
                                        if let Packet::Pong(pong) = packet {
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

                                            proto_connection.state = ProtoState::PendingJoinRequest;
                                        } else {
                                            return Err(Error::UnexpectedPacket { expected: "pong", received: packet })
                                        }
                                    } else {
                                        // we haven't received a full packet yet.
                                    }
                                },
                                ProtoState::PendingJoinRequest  => (),
                                ProtoState::PendingJoinResponse => {
                                    if let Some(packet) = proto_connection.connection.receive_packet()? {
                                        if let Packet::JoinResponse(join_response) = packet {
                                            proto_connection.state = ProtoState::Complete { join_response: join_response };
                                        } else {
                                            return Err(Error::UnexpectedPacket { expected: "join response", received: packet })
                                        }
                                    }
                                },
                                ProtoState::Complete { .. } => {
                                    // nothing to do
                                },
                            },
                            State::Connected { ref mut node, ref mut pending_connections } => {
                                let mut pending_connection = pending_connections.entry(token).unwrap();
                                pending_connection.get_mut().process_incoming_data(node)?;
                            },
                            State::Unconnected => unreachable!(),
                        }
                    },
                }
            }
        }
    }

    fn mutate_state<F>(&mut self, mut f: F) -> Result<(), Error>
        where F: FnMut(&mut Self, State) -> Result<State, Error> {
        let mut state = State::Unconnected;
        std::mem::swap(&mut state, &mut self.state);

        self.state = f(self, state)?;

        Ok(())
    }
}

fn main() {
    use clap::{App, Arg};

    let matches = App::new("parapet")
                      // .version("1.0")
                      .author("Dylan <dylanmckay34@gmail.com>")
                      .about("Peer-to-peer build system")
                      .after_help(DESCRIPTION)
                      .arg(Arg::with_name("address")
                           .help("The address of an existing node on a network to connect to")
                           .index(1))
                      .get_matches();

    let mut parapet = if let Some(address) = matches.value_of("address") {
        println!("connecting to existing network on {}", address);

        Parapet::connect(address).unwrap()
    } else {
        println!("running new network on {}:{}", SERVER_ADDRESS.0, SERVER_ADDRESS.1);

        // Create a new network.
        Parapet::new(SERVER_ADDRESS).unwrap()
    };

    parapet.run().unwrap();
}

