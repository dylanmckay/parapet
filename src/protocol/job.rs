
define_composite_type!(Run {
    executable: String,
    arguments: Vec<String>
});

define_packet_kind!(Task: u8 {
    0x00 => Run
});

define_packet!(JobRequest {
    tasks: Vec<Task>
});

