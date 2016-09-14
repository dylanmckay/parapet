#![feature(question_mark)]
#![feature(associated_consts)]
#![feature(const_fn)]
#![feature(conservative_impl_trait)]

extern crate mio;
extern crate slab;
extern crate uuid;
extern crate byteorder;
extern crate graphsearch;
extern crate dot;
#[macro_use]
extern crate protocol as proto;

pub use self::proto_connection::{ProtoConnection, ProtoState};
pub use self::connection::*;
pub use self::network::{Network, Node, Edge, Path};
pub use self::error::Error;
pub use self::protocol::{Packet, PacketKind};
pub use self::interactive::Interactive;
pub use self::job::Job;
pub use self::local::Parapet;

pub mod proto_connection;
pub mod connection;
pub mod network;
pub mod error;
pub mod protocol;
pub mod graphviz;
pub mod interactive;
pub mod job;
pub mod local;

pub const SERVER_PORT: u16 = 53371;
pub const SERVER_ADDRESS: (&'static str, u16) = ("127.0.0.1", SERVER_PORT);

pub const CLIENT_NAME: &'static str = "vanilla";
pub const CLIENT_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const PROTOCOL_MAJOR: u16 = 0;
pub const PROTOCOL_REVISION: u16 = 0;

pub fn user_agent() -> protocol::UserAgent {
    protocol::UserAgent {
        client: format!("{} v{}", CLIENT_NAME, CLIENT_VERSION),
        protocol_major: PROTOCOL_MAJOR,
        protocol_revision: PROTOCOL_REVISION,
    }
}

