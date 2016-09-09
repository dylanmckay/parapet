#![feature(question_mark)]

extern crate mio;
extern crate slab;
extern crate bincode;
extern crate rustc_serialize;
extern crate uuid;

use mio::tcp::*;
use mio::Ready;
use slab::Slab;

use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};
use self::packet::*;
use self::node::*;

mod packet;
mod node;

const SERVER_TOKEN: mio::Token = mio::Token(0);

pub struct Parapet
{
    pub listener: TcpListener,
    pub pending_connections: Slab<Connection>,
    pub nodes: Slab<Node>,
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

                        self.pending_connections.insert(Connection {
                            socket: socket,
                        }).ok();
                    },
                    _token => {
                        assert_eq!(event.kind().is_readable(), true);

                        return Ok(());
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

