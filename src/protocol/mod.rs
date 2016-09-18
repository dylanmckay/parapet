pub use self::network::{Network, Node, Edge};
pub use self::user_agent::UserAgent;
pub use self::job::{JobRequest, JobResponse, Task};

pub mod network;
pub mod user_agent;
pub mod job;

use network::Path;
use uuid::Uuid;

// Ping a node with some information.
define_packet!(Ping {
    user_agent: UserAgent,
    data: Vec<u8>
});

// Response to a ping.
define_packet!(Pong {
    user_agent: UserAgent,
    data: Vec<u8>
});

// Immediately disconnect.
define_packet!(Terminate {
    reason: String
});

// Request to join a network.
define_packet!(JoinRequest);

// Response for a network join request.
define_packet!(JoinResponse {
    your_uuid: Uuid,
    my_uuid: Uuid,
    network: Network
});

define_packet!(Packet {
    path: Path,
    kind: PacketKind
});

// Defines a packet kind enum.
define_packet_kind!(PacketKind: u32 {
    0x00 => Ping,
    0x01 => Pong,
    0x05 => Terminate,
    0x10 => JoinRequest,
    0x11 => JoinResponse,
    0x40 => JobRequest,
    0x41 => JobResponse
});

impl Packet
{
    pub fn origin(&self) -> Uuid {
        self.path.head().unwrap().clone()
    }

    pub fn destination(&self) -> Uuid {
        self.path.tail().unwrap().clone()
    }

    /// Checks if the packet is intended for a node.
    pub fn is_recipient(&self, uuid: &Uuid) -> bool {
        self.path.ends_at(uuid)
    }
}

