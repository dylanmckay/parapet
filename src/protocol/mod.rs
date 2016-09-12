pub use self::network::{Network, Node, Edge};

pub mod network;

use uuid::Uuid;

define_packet!(Ping {
    data: Vec<u8>
});

define_packet!(Pong {
    data: Vec<u8>
});

define_packet!(JoinRequest);

define_packet!(JoinResponse {
    your_uuid: Uuid,
    my_uuid: Uuid,
    network: Network
});

// Defines a packet kind enum.
define_packet_kind!(Packet: u32 {
    0x00 => Ping,
    0x01 => Pong,
    0x02 => JoinRequest,
    0x03 => JoinResponse
});

