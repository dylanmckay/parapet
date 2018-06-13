use Error;
use network::local;

use mio;
use mio::net::*;
use std;

/// Create a new tcp listener locally.
pub fn bind<A>(poll: &mio::Poll, addr: A) -> Result<TcpListener, Error>
    where A: std::net::ToSocketAddrs {
    let mut addresses = addr.to_socket_addrs()?;
    let address = addresses.next().expect("could not resolve address");

    let listener = TcpListener::bind(&address)?;

    poll.register(&listener, local::node::SERVER_TOKEN, mio::Ready::readable(),
        mio::PollOpt::edge())?;

    Ok(listener)
}

