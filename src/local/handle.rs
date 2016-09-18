use {Packet, PacketKind};
use {local, job};

pub fn packet(node: &mut local::connected::Node, packet: &Packet) {
    match packet.kind {
        PacketKind::JobRequest(ref job_request) => {
            let job: job::Job = job_request.clone().into();
            node.builder.build(packet.origin(), job);
        },
        PacketKind::JobResponse(ref job_response) => {
            println!("job completed on Node({})", packet.origin());

            for task in job_response.tasks.iter() {
                let output = String::from_utf8(task.output.clone()).unwrap();
                println!("{}", output);
            }
        },
        ref pkt => println!("received packet: {:#?}", pkt),
    }
}

