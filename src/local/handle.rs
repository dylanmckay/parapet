use {Packet, PacketKind};
use {local, job};

pub fn packet(node: &mut local::connected::Node, packet: &Packet) {
    match packet.kind {
        PacketKind::WorkRequest(ref work_request) => {
            let work: job::Work = work_request.clone().into();
            node.builder.build(packet.origin(), work);
        },
        PacketKind::WorkResponse(ref work_response) => {
            println!("job completed on Node({})", packet.origin());

            for task in work_response.tasks.iter() {
                let output = String::from_utf8(task.output.clone()).unwrap();
                println!("{}", output);
            }
        },
        ref pkt => println!("received packet: {:#?}", pkt),
    }
}

