use {Network, Packet, PacketKind, PendingState, Builder, Dispatcher, Error};
use {remote, network, protocol};

use uuid::Uuid;
use mio::tcp::*;

/// A local node that is connected to the network.
pub struct Node
{
    pub uuid: Uuid,
    pub listener: Option<TcpListener>,

    /// The network we are apart of.
    pub network: Network,

    pub builder: Builder,
    pub dispatcher: Dispatcher,
}

impl Node
{
    pub fn send_packet_to(&mut self, to: &Uuid, packet: &PacketKind) -> Result<(), Error> {
        assert!(to != &self.uuid, "can't send a packet to yourself");

        let packet = Packet {
            path: self.network.route(&self.uuid, to),
            kind: packet.clone(),
        };

        let first_hop_uuid = packet.path.next_hop(&self.uuid).expect("cannot find thing");
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

    pub fn tick(&mut self) -> Result<(), Error> {
        self.builder.tick();

        let completed_jobs: Vec<_> = self.builder.completed_jobs().collect();
        for job in completed_jobs {
            let response = PacketKind::WorkResponse(protocol::WorkResponse {
                uuid: job.output.job.uuid,
                tasks: job.output.task_results.into_iter().map(|a| protocol::job::TaskResult::from_task_result(&a)).collect(),
            });

            self.send_packet_to(&job.origin, &response)?;
        }

        Ok(())
    }

    pub fn is_listening(&self) -> bool { self.listener.is_some() }
}
