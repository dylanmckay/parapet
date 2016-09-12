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

use mio::tcp::*;
use slab::Slab;

pub use self::node::*;
pub use self::network::Network;
pub use self::error::Error;
pub use self::protocol::Packet;

pub mod node;
pub mod network;
pub mod error;
pub mod protocol;

const SERVER_TOKEN: mio::Token = mio::Token(0);
const SERVER_PORT: u16 = 53371;

pub struct Parapet
{
    pub listener: TcpListener,
    pub pending_connections: Slab<Connection, mio::Token>,
    pub network: Network,
}

impl Parapet
{
    pub fn bind<A>(addr: A) -> Result<Self, std::io::Error>
        where A: std::net::ToSocketAddrs {
        let mut addresses = addr.to_socket_addrs()?;
        let address = addresses.next().expect("could not resolve address");

        let listener = TcpListener::bind(&address)?;
        Ok(Parapet {
            listener: listener,
            pending_connections: Slab::with_capacity(1024),
            network: Network::new(),
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let poll = mio::Poll::new()?;
        poll.register(&self.listener, SERVER_TOKEN, mio::Ready::readable(),
            mio::PollOpt::edge())?;

        println!("running server");

        // Create storage for events
        let mut events = mio::Events::with_capacity(1024);

        loop {
            poll.poll(&mut events, None).unwrap();

            let mut new_nodes = Vec::new();

            for event in events.iter() {
                match event.token() {
                    SERVER_TOKEN => {
                        // Accept and drop the socket immediately, this will close
                        // the socket and notify the client of the EOF.
                        let (socket, addr) = self.listener.accept()?;

                        println!("accepted connection from {:?}", addr);

                        let entry = self.pending_connections.vacant_entry().expect("ran out of connections");
                        let token = entry.index();

                        poll.register(&socket, token, mio::Ready::readable(),
                            mio::PollOpt::edge())?;

                        entry.insert(Connection {
                            token: token,
                            protocol: proto::wire::stream::Connection::new(socket, proto::wire::middleware::pipeline::default()),
                        });
                    },
                    token => {
                        assert_eq!(event.kind().is_readable(), true);

                        if let Some(mut pending_connection) = self.pending_connections.entry(token) {
                            pending_connection.get_mut().process_incoming_data()?;

                            if let Some(packet) = pending_connection.get_mut().receive_packet()? {
                                match packet {
                                    Packet::Hello(ref hello) => {
                                        let connection = pending_connection.remove();

                                        self.network.insert(Node {
                                            connection: Some(connection),
                                            uuid: hello.uuid,
                                        });

                                        for sibling in hello.sibling_uuids.iter() {
                                            self.network.connect(hello.uuid, sibling.clone());
                                        }
                                    },
                                    Packet::Ping(..) => unimplemented!(),
                                    Packet::Pong(..) => unimplemented!(),
                                }
                            }
                        } else if let Some(mut node) = self.network.lookup_token_mut(token) {
                            node.connection.as_mut().unwrap().process_incoming_data()?;

                            if let Some(packet) = node.connection.as_mut().unwrap().receive_packet()? {
                                match packet {
                                    // Adding a new node to the network, but not directly connected
                                    // to us.
                                    Packet::Hello(ref hello) => {
                                        new_nodes.push(Node {
                                            uuid: hello.uuid,
                                            connection: None,
                                        });
                                    },
                                    Packet::Ping(..) => unimplemented!(),
                                    Packet::Pong(..) => unimplemented!(),
                                }
                            }
                        } else {
                            unreachable!()
                        }
                    },
                }
            }

            for node in new_nodes.drain(..) { self.network.insert(node); }
        }
    }
}

fn main() {
    let mut parapet = Parapet::bind(("127.0.0.1", SERVER_PORT)).unwrap();
    parapet.run().ok();
}

