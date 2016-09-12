#![feature(question_mark)]
#![feature(associated_consts)]
#![feature(conservative_impl_trait)]

extern crate mio;
extern crate slab;
extern crate uuid;
extern crate byteorder;
extern crate graphsearch;
#[macro_use]
extern crate protocol as proto;

pub use self::connection::*;
pub use self::network::{Network, Node, Edge};
pub use self::error::Error;
pub use self::protocol::Packet;
pub use self::path::Path;

pub mod server;
pub mod connection;
pub mod network;
pub mod error;
pub mod protocol;
pub mod path;

use mio::tcp::*;
use slab::Slab;

use uuid::Uuid;

const SERVER_TOKEN: mio::Token = mio::Token(0);

const SERVER_PORT: u16 = 53371;
const SERVER_ADDRESS: (&'static str, u16) = ("127.0.0.1", SERVER_PORT);

// Flow:
//
// Client sends 'JoinRequest' to some node
// Server responds with 'JoinResponse'
// Client is now ready to act as server.

pub enum State
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
    /// We are now a connected node in the network.
    Connected {
        uuid: Uuid,
        listener: TcpListener,

        pending_connections: Slab<server::ProtoConnection, mio::Token>,

        /// The network we are apart of.
        network: Network,
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
                uuid: Uuid::new_v4(),
                listener: listener,
                pending_connections: Slab::with_capacity(1024),
                network: Network::new(),
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

    pub fn mutate_state<F>(&mut self, mut f: F) -> Result<(), Error>
        where F: FnMut(&mut Self, State) -> Result<State, Error> {
        // TODO: remove this dirty hack. it is required because we
        // can't move `state` out of the borrowed `self`.
        let mut state = unsafe { std::mem::uninitialized() };
        std::mem::swap(&mut state, &mut self.state);

        self.state = f(self, state)?;

        Ok(())
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
            state: State::PendingPing {
                connection: Connection {
                    token: tmp_token,
                    protocol: proto::wire::stream::Connection::new(stream, proto::wire::middleware::pipeline::default()),
                },
            },
            poll: poll,
        })
    }

    /// Attempts to advance the current state if possible.
    pub fn advance(&mut self) -> Result<(), Error> {
        self.mutate_state(|_, state| match state {
            State::PendingPing { mut connection } => {
                let ping = protocol::Ping {
                    // TODO: randomise this data
                    data: vec![6, 2, 6, 1, 8, 8],
                };

                connection.send_packet(&Packet::Ping(ping.clone()))?;

                Ok(State::PendingPong { original_ping: ping, connection: connection })
            },
            State::PendingJoinRequest { mut connection } => {
                connection.send_packet(&Packet::JoinRequest(protocol::JoinRequest))?;

                Ok(State::PendingJoinResponse { connection: connection })
            },
            state => Ok(state), // we don't have to do anything.
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        println!("running server");

        // Create storage for events
        let mut events = mio::Events::with_capacity(1024);

        loop {
            self.poll.poll(&mut events, None).unwrap();

            self.advance()?;

            for event in events.iter() {
                match event.token() {
                    // A pending connection.
                    SERVER_TOKEN => {
                        if let State::Connected { ref mut pending_connections, ref mut listener, .. } = self.state {
                            // Accept and drop the socket immediately, this will close
                            // the socket and notify the client of the EOF.
                            let (socket, addr) = listener.accept()?;

                            println!("accepted connection from {:?}", addr);

                            let entry = pending_connections.vacant_entry().expect("ran out of connections");
                            let token = entry.index();

                            self.poll.register(&socket, token, mio::Ready::readable(),
                                mio::PollOpt::edge())?;

                            entry.insert(server::ProtoConnection::new(Connection {
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
                        assert_eq!(event.kind().is_readable(), true);

                        // We have received data from a node.
                        self.mutate_state(|parapet, state| match state {
                            State::Connected { uuid, mut pending_connections, listener, network } => {
                                if let Some(mut pending_connection) = pending_connections.entry(token) {
                                    pending_connection.get_mut().process_incoming_data()?;

                                    // promote connection to node if possible.
                                    unimplemented!();
                                }

                                Ok(State::Connected {
                                    uuid: uuid,
                                    pending_connections: pending_connections,
                                    listener: listener,
                                    network: network,
                                })
                            },
                            State::PendingPing { mut connection } => {
                                connection.process_incoming_data()?;

                                Ok(State::PendingPing { connection: connection })
                            },
                            State::PendingPong { mut connection, original_ping } => {
                                connection.process_incoming_data()?;

                                if let Some(packet) = connection.receive_packet()? {
                                    if let Packet::Pong(pong) = packet {
                                        // Check if the echoed data is correct.
                                        if pong.data != original_ping.data {
                                            Err(Error::InvalidPong{
                                                expected: original_ping.data.clone(),
                                                received: pong.data,
                                            })
                                        } else {
                                            Ok(State::PendingJoinRequest { connection: connection })
                                        }
                                    } else {
                                        Err(Error::UnexpectedPacket { expected: "pong", received: packet })
                                    }
                                } else {
                                    // we haven't received a full packet yet.
                                    Ok(State::PendingPong { connection: connection, original_ping: original_ping })
                                }
                            },
                            State::PendingJoinRequest { mut connection } => {
                                connection.process_incoming_data()?;

                                Ok(State::PendingJoinRequest { connection: connection })
                            },
                            State::PendingJoinResponse { mut connection } => {
                                connection.process_incoming_data()?;

                                if let Some(packet) = connection.receive_packet()? {
                                    if let Packet::JoinResponse(join_response) = packet {
                                        let listener = Parapet::bind(&mut parapet.poll, SERVER_ADDRESS)?;

                                        Ok(State::Connected {
                                            uuid: join_response.your_uuid,
                                            listener: listener,
                                            pending_connections: Slab::with_capacity(1024),
                                            network: join_response.network.into(),
                                        })
                                    } else {
                                        Err(Error::UnexpectedPacket { expected: "join response", received: packet })
                                    }
                                } else {
                                    Ok(State::PendingJoinResponse { connection: connection })
                                }
                            },
                        })?;
                    },
                }
            }
        }
    }
}

fn main() {
    // Create a new network.
    let mut parapet = Parapet::new(SERVER_ADDRESS).unwrap();
    parapet.run().ok();
}

