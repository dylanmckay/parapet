use {Error, Network};
use {network, ci};
use network::Connection;

use mio;
use mio::tcp::*;
use slab::Slab;
use uuid::Uuid;

use std;
use proto;

pub struct Parapet
{
    pub node: network::local::Node,
    pub poll: mio::Poll,
}

impl Parapet
{
    /// Create a new network.
    pub fn new<A>(addr: A) -> Result<Self, Error>
        where A: std::net::ToSocketAddrs {
        let mut poll = mio::Poll::new()?;

        let listener = network::local::tcp::bind(&mut poll, addr)?;
        let uuid = Uuid::new_v4();

        println!("assigning UUID {}", uuid);

        Ok(Parapet {
            node: network::local::Node::Connected {
                node: network::local::connected::Node {
                    uuid: uuid,
                    listener: Some(listener),
                    network: Network::new(uuid),
                    notify: network::Notify::new(),
                    builder: ci::Builder::new(),
                    dispatcher: ci::Dispatcher::new(),
                },
                pending_connections: Slab::with_capacity(1024),
            },
            poll: poll,
        })
    }

    /// Connect to an existing network.
    /// * `addr` - Any node on the network.
    pub fn connect<A>(addr: A) -> Result<Self, std::io::Error>
        where A: std::net::ToSocketAddrs {
        let mut addresses = addr.to_socket_addrs()?;
        let address = addresses.next().expect("could not resolve address");

        let stream = TcpStream::connect(&address)?;

        let poll = mio::Poll::new()?;
        poll.register(&stream, network::local::node::NEW_CONNECTION_TOKEN, mio::Ready::writable() | mio::Ready::readable(),
            mio::PollOpt::edge())?;

        let connection = Connection {
            token: network::local::node::NEW_CONNECTION_TOKEN,
            protocol: proto::wire::stream::Connection::new(stream, proto::wire::middleware::pipeline::default()),
        };

        Ok(Parapet {
            node: network::local::Node::Pending(network::local::pending::Node::new(connection)),
            poll: poll,
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            match self.tick() {
                Ok(..) => (),
                Err(Error::Stop { reason }) => {
                    println!("stopping: {}", reason);
                    break;
                },
                e => return e,
            }
        }

        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), Error> {
        self.node.tick(&mut self.poll)
    }
}
