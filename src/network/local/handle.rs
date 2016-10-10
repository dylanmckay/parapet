use {Packet, PacketKind};
use ci;
use network::{self, local};

pub fn packet(node: &mut local::connected::Node, packet: &Packet) {
    let sender_uuid = packet.path.sender();
    let sender = node.network.get_mut(&sender_uuid).unwrap();

    match packet.kind {
        PacketKind::WorkRequest(ref work_request) => {
            let work: ci::build::Work = work_request.clone().into();
            node.builder.build(packet.origin(), work);
        },
        PacketKind::WorkResponse(ref work_response) => {
            println!("job completed on Node({})", packet.origin());

            for task in work_response.tasks.iter() {
                let output = String::from_utf8(task.output.clone()).unwrap();
                println!("{}", output);
            }
        },
        PacketKind::WorkAvailable(..) => {
            if let network::Status::Remote(ref mut status) = sender.status {
                status.work_available = true;
            }
        },
        PacketKind::WorkComplete(..) => {
            if let network::Status::Remote(ref mut status) = sender.status {
                status.work_available = false;
            }
        },
        ref pkt => println!("received packet: {:#?}", pkt),
    }
}

