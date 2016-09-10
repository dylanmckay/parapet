#![feature(question_mark)]
#![feature(associated_consts)]
#![feature(custom_derive, plugin)]

#![plugin(serde_macros)]

extern crate mio;
extern crate slab;
extern crate uuid;
extern crate byteorder;
extern crate serde;
extern crate serde_json;

use mio::tcp::*;
use slab::Slab;

pub use self::packet::*;
pub use self::node::*;

pub mod packet;
pub mod node;
pub mod io;

const SERVER_TOKEN: mio::Token = mio::Token(0);

pub struct Parapet
{
    pub listener: TcpListener,
    pub pending_connections: Slab<Connection, mio::Token>,
    pub nodes: Slab<Node, mio::Token>,
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
            nodes: Slab::with_capacity(1024),
        })
    }

    pub fn run(&mut self) -> Result<(), std::io::Error> {
        let poll = mio::Poll::new()?;
        poll.register(&self.listener, SERVER_TOKEN, mio::Ready::readable(),
            mio::PollOpt::edge())?;

        println!("running server");

        // Create storage for events
        let mut events = mio::Events::with_capacity(1024);

        loop {
            poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                match event.token() {
                    SERVER_TOKEN => {
                        // Accept and drop the socket immediately, this will close
                        // the socket and notify the client of the EOF.
                        let (socket, addr) = self.listener.accept()?;

                        println!("accepted connection from {:?}", addr);

                        let entry = self.pending_connections.vacant_entry().expect("ran out of connections");

                        poll.register(&socket, entry.index(), mio::Ready::readable(),
                            mio::PollOpt::edge())?;

                        entry.insert(Connection {
                            socket: socket,
                            builder: io::Builder::new(),
                        });
                    },
                    token => {
                        assert_eq!(event.kind().is_readable(), true);

                        if let Some(mut pending_connection) = self.pending_connections.entry(token) {
                            pending_connection.get_mut().process_incoming_data()?;

                            if let Some(packet) = pending_connection.get_mut().take_packet()? {
                                match packet {
                                    Packet::Hello(ref hello) => {
                                        let connection = pending_connection.remove();

                                        self.nodes.insert(Node {
                                            connection: Some(connection),
                                            uuid: hello.uuid,
                                        }).ok();
                                    },
                                }
                            }
                        } else if let Some(mut node) = self.nodes.entry(token) {
                            node.get_mut().connection.as_mut().unwrap().process_incoming_data()?;

                            if let Some(packet) = node.get_mut().connection.as_mut().unwrap().take_packet()? {
                                match packet {
                                    // Adding a new node to the network, but not directly connected
                                    // to us.
                                    Packet::Hello(ref hello) => {
                                        self.nodes.insert(Node {
                                            uuid: hello.uuid,
                                            connection: None,
                                        });
                                    },
                                }
                            }
                        } else {
                            unreachable!()
                        }
                    },
                }
            }
        }
    }
}

fn main() {
    let mut parapet = Parapet::bind("127.0.0.1:45722").unwrap();
    parapet.run().unwrap();
}

