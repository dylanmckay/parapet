use {Error, Connection, PendingState, Builder, Dispatcher};
use {local, remote, network, workspace};

use slab::Slab;
use proto;
use mio;
use std;

use std::time::Duration;

pub const SERVER_TOKEN: mio::Token = mio::Token(usize::max_value() - 10);
pub const NEW_CONNECTION_TOKEN: mio::Token = mio::Token(usize::max_value() - 11);

pub enum Node
{
    Unconnected,
    Pending(local::pending::Node),
    /// We are now a connected node in the network.
    Connected {
        node: local::connected::Node,

        pending_connections: Slab<remote::pending::Node, mio::Token>,
    },
}

impl Node
{
    pub fn tick(&mut self, poll: &mut mio::Poll) -> Result<(), Error> {
        // Create storage for events
        let mut events = mio::Events::with_capacity(1024);

        self.try_complete_pending_connection(poll)?;
        poll.poll(&mut events, Some(Duration::from_millis(10))).unwrap();

        if let Node::Connected { ref mut node, .. } = *self {
            node.tick()?;

            // If we've been orphaned, and we aren't listening locally.
            if !node.is_listening() && node.network.node_count() == 1 {
                return Err(Error::Stop { reason: "local node has stopped, and we aren't bound to local port, so we are orphaned" })
            }
        }

        for event in events.iter() {
            match event.token() {
                // A pending connection.
                SERVER_TOKEN => {
                    if let local::Node::Connected { ref mut node, ref mut pending_connections } = *self {
                        let (socket, addr) = node.listener.as_mut().unwrap().accept()?;

                        println!("accepted connection from {:?}", addr);

                        let entry = pending_connections.vacant_entry().expect("ran out of connections");
                        let token = entry.index();

                        poll.register(&socket, token, mio::Ready::readable() | mio::Ready::writable(),
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

                    match *self {
                        local::Node::Pending(ref mut pending_node) => {
                            assert_eq!(token, NEW_CONNECTION_TOKEN);

                            if event.kind().is_hup() {
                                println!("remote node disconnected before we could join");
                                return Ok(());
                            } else if !event.kind().is_readable() {
                                continue;
                            }

                            pending_node.process_incoming_data()?;
                        },
                        local::Node::Connected { ref mut node, ref mut pending_connections } => {
                            if !event.kind().is_readable() {
                                continue;
                            }

                            let packet = if let Some(mut pending_connection) = pending_connections.entry(token) {
                                if event.kind().is_hup() {
                                    println!("pending remote connection disconnected");
                                    pending_connection.remove();
                                } else if event.kind().is_readable() {
                                    pending_connection.get_mut().process_incoming_data(node)?;

                                    if pending_connection.get().is_complete() {
                                        node.promote_pending_connection_to_node(pending_connection.remove()).unwrap();
                                    }
                                }

                                continue;
                            } else if let Some(mut from_node) = node.network.entry_by_token(token) {
                                // we received a packet from an established node

                                if event.kind().is_hup() {
                                    println!("node {} disconnected", from_node.get().uuid);
                                    from_node.remove();

                                    continue;
                                } else if event.kind().is_readable() {
                                    if let Some(packet) = from_node.get_mut().connection.as_mut().unwrap().receive_packet()? {
                                        packet
                                    } else {
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            } else {
                                unreachable!();
                            };

                            // Check if the packet is for us.
                            if packet.is_recipient(&node.uuid) {
                                local::handle::packet(node, &packet);
                            } else {
                                // we need to forward this packet to the recipient
                                let next_hop_uuid = packet.path.next_hop(&node.uuid).unwrap();
                                let next_hop = node.network.get_mut(&next_hop_uuid).unwrap();

                                next_hop.connection.as_mut().unwrap().send_packet(&packet)?;
                            }
                        },
                        local::Node::Unconnected => unreachable!(),
                    }
                },
            }
        }

        Ok(())
    }

    /// Attempts to advance the current state if possible.
    pub fn try_complete_pending_connection(&mut self, poll: &mut mio::Poll) -> Result<(), Error> {
        let mut current_node = local::Node::Unconnected;
        std::mem::swap(&mut current_node, self);

        *self = match current_node {
            local::Node::Pending(mut node) => {
                node.advance_state()?;

                if let PendingState::Complete { join_response } = node.state.clone() {
                    let listener = match local::tcp::bind(poll, ::SERVER_ADDRESS) {
                        Ok(listener) => Some(listener),
                        Err(Error::Io(e)) => match e.kind() {
                            std::io::ErrorKind::AddrInUse => {
                                println!("there is already something listening on port {} - we're not going to listen", ::SERVER_PORT);
                                None
                            },
                            _ => return Err(Error::Io(e)),
                        },
                        Err(e) => return Err(e),
                    };

                    let mut network: network::Network = join_response.network.into();
                    network.set_connection(&join_response.my_uuid, node.connection);

                    // Add ourselves to the network.
                    network.insert(network::Node {
                        uuid: join_response.your_uuid,
                        connection: None,
                    });

                    // Connect ourselves and the node we're communicating with.
                    network.connect(&join_response.your_uuid, &join_response.my_uuid);

                    println!("connected to network with UUID {}", join_response.your_uuid);

                    let builder = Builder::new(Box::new(
                        workspace::strategy::InDirectory::<workspace::Basic>::new("parapet-cache"))
                    );


                    local::Node::Connected {
                        node: local::connected::Node {
                            uuid: join_response.your_uuid,
                            listener: listener,
                            network: network,
                            builder: builder,
                            dispatcher: Dispatcher::new(),
                        },
                        pending_connections: Slab::with_capacity(1024),
                    }
                } else {
                    local::Node::Pending(node)
                }
            },
            node => node,
        };

        Ok(())
    }
}

