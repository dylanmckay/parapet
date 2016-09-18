use {Network, Packet, PacketKind, PendingState, Error};
use {remote, network};

use uuid::Uuid;
use mio::tcp::*;

/// A local node that is connected to the network.
pub struct Node
{
    pub uuid: Uuid,
    pub listener: Option<TcpListener>,

    /// The network we are apart of.
    pub network: Network,
}

impl Node
{
    pub fn send_packet_to(&mut self, to: &Uuid, packet: &PacketKind) -> Result<(), Error> {
        assert!(to != &self.uuid, "can't send a packet to yourself");

        let packet = Packet {
            path: self.network.route(&self.uuid, to),
            kind: packet.clone(),
        };

        println!("route: {:?}", packet.path);

        println!("my uuid: {}", self.uuid);
        let first_hop_uuid = packet.path.next_hop(&self.uuid).expect("cannot find thing");
        println!("first hop: {:?}", first_hop_uuid);
        let first_hop = self.network.get_mut(&first_hop_uuid).expect("can't find first hop from uuid");
        first_hop.connection.as_mut().unwrap().send_packet(&packet)
    }

    pub fn broadcast_packet(&mut self, packet: &PacketKind) -> Result<(), Error> {
        let destination_uuids: Vec<_> = self.network.nodes()
            .filter(|node| node.uuid != self.uuid)
            .map(|node| node.uuid.clone())
            .collect();

        for destination_uuid in destination_uuids {
            self.send_packet_to(&destination_uuid, packet)?;
        }

        Ok(())
    }

    pub fn promote_pending_connection_to_node(&mut self, pending_connection: remote::pending::Node) -> Result<(), Error> {
        if let PendingState::Complete { join_response } = pending_connection.state {
            self.network.insert(network::Node {
                uuid: join_response.your_uuid,
                connection: Some(pending_connection.connection),
            });

            self.network.connect(&self.uuid, &join_response.your_uuid);
        } else {
            unreachable!();
        }

        Ok(())
    }
}
