use {Error, Packet, PacketKind};
use {ci, protocol};
use network::{self, local, remote};

pub fn packet(node: &mut local::connected::Node, packet: &Packet)
    -> Result<(), Error> {
    match packet.kind {
        PacketKind::WorkRequest(..) => {
            if let Some(work) = node.dispatcher.poll() {
                println!("giving data");
                node.send_packet_to(&packet.origin(), &PacketKind::WorkResponse(protocol::WorkResponse::from_work(&work)))?
            }

            Ok(())
        },
        PacketKind::WorkResponse(ref work_response) => {
            let work: ci::build::Work = work_response.clone().into();
            node.builder.build(packet.origin(), work);

            Ok(())
        },
        PacketKind::WorkFinished(ref work_finished) => {
            println!("job completed on Node({})", packet.origin());

            for task in work_finished.tasks.iter() {
                let output = String::from_utf8(task.output.clone()).unwrap();
                println!("{}", output);
            }

            node.dispatcher.complete(ci::dispatcher::CompletedWork {
                uuid: work_finished.uuid,
                task_results: work_finished.tasks.iter().cloned().map(|a| a.into()).collect(),
            });

            Ok(())
        },
        PacketKind::WorkAvailable(..) => {
            let sender_uuid = packet.path.sender();
            let sender = node.network.get_mut(&sender_uuid).unwrap();

            println!("work available on {}", sender_uuid);

            if let network::Status::Remote(ref mut status) = sender.status {
                status.work = remote::status::Work::Available { have_asked_for_work: false };
            }

            Ok(())
        },
        PacketKind::WorkComplete(..) => {
            let sender_uuid = packet.path.sender();
            let sender = node.network.get_mut(&sender_uuid).unwrap();

            if let network::Status::Remote(ref mut status) = sender.status {
                status.work = remote::status::Work::Unavailable;
            }

            Ok(())
        },
        ref pkt => {
            println!("received packet: {:#?}", pkt);
            Ok(())
        },
    }
}

