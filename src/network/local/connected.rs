use {Network, Packet, PacketKind, Error};
use network::{remote, PendingState, Notify};
use {network, protocol, ci};

use uuid::Uuid;
use mio::tcp::*;

/// A local node that is connected to the network.
pub struct Node
{
    pub uuid: Uuid,
    pub listener: Option<TcpListener>,

    /// The network we are apart of.
    pub network: Network,

    pub notify: Notify,
    pub builder: ci::Builder,
    pub dispatcher: ci::Dispatcher,
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
                status: network::Status::Remote(network::remote::Status::default()),
            });

            self.network.connect(&self.uuid, &join_response.your_uuid);
        } else {
            unreachable!();
        }

        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), Error> {
        self.builder.tick();

        if self.dispatcher.has_work() { self.notify.work.available() } else { self.notify.work.complete() }

        for packet in self.notify.notify() {
            self.broadcast_packet(&packet)?;
        }

        self.ask_for_work()?;

        let completed_work: Vec<_> = self.builder.completed_work().collect();
        for work in completed_work {
            let response = PacketKind::WorkFinished(protocol::WorkFinished {
                uuid: work.output.work.uuid,
                tasks: work.output.task_results.into_iter().map(|a| protocol::ci::TaskResult::from_task_result(&a)).collect(),
            });

            self.send_packet_to(&work.origin, &response)?;
        }

        Ok(())
    }

    pub fn is_listening(&self) -> bool { self.listener.is_some() }

    fn ask_for_work(&mut self) -> Result<(), Error> {
        // If we're feeling up to it, grab some work from other nodes.
        if self.builder.should_pickup_work() {
            let node_uuid = self.network.nodes().filter(|n| n.can_ask_for_work()).next().map(|n| n.uuid.clone());

            if let Some(node_uuid) = node_uuid {
                self.send_packet_to(&node_uuid, &PacketKind::WorkRequest(protocol::WorkRequest))?;

                if let remote::status::Work::Available { ref mut have_asked_for_work }
                    = self.network.get_mut(&node_uuid).unwrap().status.expect_remote_mut().work {
                    *have_asked_for_work = true;
                }
            }
        }

        Ok(())
    }
}
