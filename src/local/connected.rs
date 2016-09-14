use {Network, Packet, PacketKind, Error};

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
        let packet = Packet {
            path: self.network.route(&self.uuid, to),
            kind: packet.clone(),
        };

        let first_hop_uuid = packet.path.next_hop(&self.uuid).unwrap();
        let first_hop = self.network.get_mut(&first_hop_uuid).unwrap();
        first_hop.connection.as_mut().unwrap().send_packet(&packet)
    }

    pub fn broadcast_packet(&mut self, packet: &PacketKind) -> Result<(), Error> {
        let destination_uuids: Vec<_> = self.network.nodes()
            .filter(|node| node.uuid == self.uuid)
            .map(|node| node.uuid.clone())
            .collect();

        for destination_uuid in destination_uuids {
            self.send_packet_to(&destination_uuid, packet)?;
        }

        Ok(())
    }
}
